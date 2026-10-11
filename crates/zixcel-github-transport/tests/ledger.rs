use zixcel_github_egress_contracts::GitHubEgressReceiptV1;
use zixcel_github_transport::{ExecutionLedger, ExecutionStart};

#[test]
fn prepared_is_ambiguous_completed_is_exact_and_substitution_fails_closed() {
    let directory = tempfile::tempdir().expect("ledger directory");
    let ledger = ExecutionLedger::new(directory.path().join("ledger.json"));
    assert_eq!(
        ledger.begin("authorization-1", "digest-1").expect("begin"),
        ExecutionStart::Execute
    );
    assert_eq!(
        ledger
            .begin("authorization-1", "digest-1")
            .expect_err("ambiguous")
            .0,
        "provider-outcome-unknown"
    );
    assert_eq!(
        ledger
            .begin("authorization-1", "digest-2")
            .expect_err("substitution")
            .0,
        "authorization-replay"
    );
    let receipt = receipt();
    ledger
        .complete("authorization-1", "digest-1", receipt.clone())
        .expect("complete");
    assert_eq!(
        ledger.begin("authorization-1", "digest-1").expect("replay"),
        ExecutionStart::Completed(receipt)
    );
    assert_eq!(
        ledger
            .begin("authorization-1", "digest-2")
            .expect_err("completed substitution")
            .0,
        "authorization-replay"
    );
}

fn receipt() -> GitHubEgressReceiptV1 {
    GitHubEgressReceiptV1 {
        schema: "crowsi://provider-egress/github-receipt/v1".into(),
        authorization_id: "authorization-1".into(),
        authorization_digest_sha256: "sha256:authorization".into(),
        api_request_digest_sha256: "sha256:request".into(),
        provider_request_id: "github-request-1".into(),
        remote_reference: "github://example/repository".into(),
        remote_digest_sha256: Some("sha256:remote".into()),
        completed_at_epoch_s: 100,
    }
}

// Spawn the test executable itself to exercise independent OS processes.
#[test]
fn ledger_process_worker() {
    let Ok(root) = std::env::var("LEDGER_TEST_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let worker = std::env::var("LEDGER_TEST_WORKER").expect("worker");
    let mode = std::env::var("LEDGER_TEST_MODE").expect("mode");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !root.join("start").exists() {
        assert!(std::time::Instant::now() < deadline, "start timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let ledger = ExecutionLedger::new(root.join("ledger.json"));
    let id = if mode == "same" { "same" } else { &worker };
    let result = ledger.begin(id, "digest");
    if mode == "hold" {
        assert_eq!(
            result.expect("reserve before crash"),
            ExecutionStart::Execute
        );
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join("ledger.json.lock"))
            .expect("sidecar");
        lock.lock().expect("lock");
        std::fs::write(root.join("ready"), "ready").expect("ready");
        loop {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    let outcome = match result {
        Ok(ExecutionStart::Execute) => {
            if mode == "different" {
                let mut receipt = receipt();
                receipt.authorization_id = id.into();
                ledger.complete(id, "digest", receipt).expect("complete");
            }
            "execute"
        }
        Err(error) if error.0 == "provider-outcome-unknown" => "unknown",
        other => panic!("unexpected worker outcome: {other:?}"),
    };
    std::fs::write(root.join(format!("result-{worker}")), outcome).expect("result");
}

fn spawn_worker(root: &std::path::Path, worker: &str, mode: &str) -> std::process::Child {
    std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "ledger_process_worker", "--nocapture"])
        .env("LEDGER_TEST_ROOT", root)
        .env("LEDGER_TEST_WORKER", worker)
        .env("LEDGER_TEST_MODE", mode)
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("spawn worker")
}

fn finish_workers(workers: &mut [std::process::Child]) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    for worker in workers {
        loop {
            if let Some(status) = worker.try_wait().expect("poll worker") {
                assert!(status.success(), "worker failed");
                break;
            }
            if std::time::Instant::now() >= deadline {
                worker.kill().expect("kill stalled worker");
                worker.wait().expect("reap stalled worker");
                panic!("worker timeout");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}

#[test]
fn competing_processes_reserve_same_authorization_once() {
    let directory = tempfile::tempdir().expect("directory");
    let mut workers: Vec<_> = (0..12)
        .map(|i| spawn_worker(directory.path(), &i.to_string(), "same"))
        .collect();
    std::fs::write(directory.path().join("start"), "go").expect("start");
    finish_workers(&mut workers);
    let outcomes: Vec<_> = (0..12)
        .map(|i| {
            std::fs::read_to_string(directory.path().join(format!("result-{i}"))).expect("result")
        })
        .collect();
    assert_eq!(
        outcomes.iter().filter(|value| *value == "execute").count(),
        1
    );
    assert_eq!(
        outcomes.iter().filter(|value| *value == "unknown").count(),
        11
    );
}

#[test]
fn competing_processes_preserve_distinct_reservations_and_completions() {
    let directory = tempfile::tempdir().expect("directory");
    let mut workers: Vec<_> = (0..12)
        .map(|i| spawn_worker(directory.path(), &i.to_string(), "different"))
        .collect();
    std::fs::write(directory.path().join("start"), "go").expect("start");
    finish_workers(&mut workers);
    let ledger = ExecutionLedger::new(directory.path().join("ledger.json"));
    for i in 0..12 {
        assert!(matches!(
            ledger.begin(&i.to_string(), "digest").expect("replay"),
            ExecutionStart::Completed(_)
        ));
    }
}

#[test]
fn killed_lock_holder_releases_lock_without_reexecuting_ambiguous_reservation() {
    let directory = tempfile::tempdir().expect("directory");
    let mut child = spawn_worker(directory.path(), "crashed", "hold");
    std::fs::write(directory.path().join("start"), "go").expect("start");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !directory.path().join("ready").exists() {
        if std::time::Instant::now() >= deadline {
            child.kill().expect("kill");
            child.wait().expect("reap");
            panic!("ready timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    child.kill().expect("kill lock holder");
    child.wait().expect("reap lock holder");
    let ledger = ExecutionLedger::new(directory.path().join("ledger.json"));
    assert_eq!(
        ledger.begin("crashed", "digest").expect_err("unknown").0,
        "provider-outcome-unknown"
    );
    assert_eq!(
        ledger.begin("new", "digest").expect("released lock"),
        ExecutionStart::Execute
    );
}
