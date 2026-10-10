use crate::{artifact_store::ArtifactStore, github_provider_action::perform};
use sha2::{Digest, Sha256};
use std::fs;
use zixcel_github::{GitHubAction, GitHubApiRequest, GitHubRemoteExpectation};
fn request(action: GitHubAction) -> GitHubApiRequest {
    GitHubApiRequest {
        schema: zixcel_github::API_REQUEST_SCHEMA.into(),
        request_id: "request-1".into(),
        operation_id: "operation-1".into(),
        connection_ref: "connection-1".into(),
        owner: "example-org".into(),
        repository: "repo".into(),
        action,
    }
}
fn artifact(root: &std::path::Path, value: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(value).expect("local provider fixture");
    let hash = hex::encode(Sha256::digest(&bytes));
    fs::write(root.join(format!("{hash}.json")), bytes).expect("local provider fixture");
    hash
}
#[test]
fn organization_create_snapshot_and_guarded_delete_follow_the_real_api_shapes() {
    let root = tempfile::tempdir().expect("local provider fixture");
    let curl = root.path().join("curl-fixture");
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/github_fixture.rs");
    let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    assert!(
        std::process::Command::new(compiler)
            .arg("--edition=2024")
            .arg(fixture)
            .arg("-o")
            .arg(&curl)
            .status()
            .expect("compile local fixture")
            .success()
    );
    let store = ArtifactStore::new(root.path().to_str().expect("local provider fixture"))
        .expect("local provider fixture");
    let state = root.path().join("state.json");
    let run = |action| {
        perform(
            &request(action),
            b"fixture",
            curl.to_str().expect("local provider fixture"),
            &state,
            &store,
        )
    };
    run(GitHubAction::CreatePrivateRepository {
        default_branch: "main".into(),
    })
    .expect("local provider fixture");
    let snapshot = artifact(
        root.path(),
        &serde_json::json!({"schema":"zixcel://github/source-snapshot-artifact/v1","branch":"main","message":"fixture","files":[{"path":"README.md","mode":"100644","content_base64":"Zml4dHVyZQ=="}]}),
    );
    run(GitHubAction::PushRepositorySnapshot {
        snapshot_ref: format!("sha256:{snapshot}"),
        snapshot_digest_sha256: snapshot,
        expected_remote: GitHubRemoteExpectation::Exact {
            commit_sha256: hex::encode(Sha256::digest("a".repeat(40).as_bytes())),
        },
    })
    .expect("local provider fixture");
    let backup = artifact(
        root.path(),
        &serde_json::json!({"schema":"zixcel://github/deletion-backup/v1","repository_id":"12345","owner":"example-org","repository":"repo","bundle_digest_sha256":format!("sha256:{}","e".repeat(64))}),
    );
    assert!(
        run(GitHubAction::DeleteRepository {
            expected_repository_id: "54321".into(),
            backup_digest_sha256: backup.clone()
        })
        .is_err()
    );
    let before =
        fs::read_to_string(root.path().join("calls.jsonl")).expect("local provider fixture");
    assert!(!before.contains("DELETE"));
    run(GitHubAction::DeleteRepository {
        expected_repository_id: "12345".into(),
        backup_digest_sha256: backup,
    })
    .expect("local provider fixture");
    let calls =
        fs::read_to_string(root.path().join("calls.jsonl")).expect("local provider fixture");
    assert!(!calls.contains("/user/repos"));
    assert_eq!(calls.matches("DELETE").count(), 1);
}
