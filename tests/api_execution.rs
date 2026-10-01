mod support;

use support::{DIGEST, RecordingBackend, request};
use zixcel_github::{
    GitHubAction, GitHubRemoteExpectation, execute_api_request, parse_api_request_json,
};

#[test]
fn valid_api_request_executes_once() {
    let request = request(GitHubAction::PushRepositorySnapshot {
        snapshot_ref: "artifact/source-snapshot-1".into(),
        snapshot_digest_sha256: DIGEST.into(),
        expected_remote: GitHubRemoteExpectation::Absent,
    });
    let mut backend = RecordingBackend::default();
    let receipt = execute_api_request(&request, &mut backend).expect("valid API request");
    assert_eq!(backend.calls(), 1);
    assert_eq!(receipt.permission.resource, "github-repository-content");
    assert_eq!(receipt.permission.operation, "publish-artifact");
    assert_eq!(receipt.remote_digest_sha256.as_deref(), Some(DIGEST));
}

#[test]
fn workflow_artifact_download_is_a_distinct_permission_from_source_push() {
    let request = request(GitHubAction::DownloadWorkflowArtifact {
        workflow_run_id: "123".into(),
        artifact_id: "456".into(),
        expected_digest_sha256: Some(DIGEST.into()),
    });
    let mut backend = RecordingBackend::default();
    let receipt = execute_api_request(&request, &mut backend).expect("valid API request");
    assert_eq!(backend.calls(), 1);
    assert_eq!(receipt.permission.resource, "github-actions-artifact");
    assert_eq!(receipt.permission.operation, "download-artifact");
}

#[test]
fn invalid_target_never_reaches_backend() {
    let mut request = request(GitHubAction::ObserveRepositoryMetadata);
    request.owner = "*".into();
    let mut backend = RecordingBackend::default();
    assert!(execute_api_request(&request, &mut backend).is_err());
    assert_eq!(backend.calls(), 0);
}

#[test]
fn wire_cannot_request_public_creation_force_push_or_caller_context() {
    let base = serde_json::to_value(request(GitHubAction::CreatePrivateRepository {
        default_branch: "main".into(),
    }))
    .expect("request JSON");
    for (name, value) in [
        ("visibility", serde_json::json!("public")),
        ("force", serde_json::json!(true)),
    ] {
        let mut changed = base.clone();
        changed["action"][name] = value;
        let bytes = serde_json::to_vec(&changed).expect("bytes");
        assert!(parse_api_request_json(&bytes).is_err(), "{name}");
    }
    let mut caller = base;
    caller["caller_package_id"] = serde_json::json!("package/example");
    assert!(parse_api_request_json(&serde_json::to_vec(&caller).expect("bytes")).is_err());
}
