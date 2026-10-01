use zixcel_github::{API_REQUEST_SCHEMA, GitHubAction, GitHubApiRequest, parse_api_request_json};

#[test]
fn api_request_has_no_caller_or_workflow_identity_surface() {
    let wire = serde_json::to_value(request()).expect("request JSON");
    for forbidden in [
        "caller_package_id",
        "caller_package_digest_sha256",
        "binding_digest_sha256",
        "invocation_id",
        "authorization_id",
    ] {
        assert!(wire.get(forbidden).is_none(), "{forbidden}");
    }
}

#[test]
fn open_api_documents_fail_closed() {
    let mut open = serde_json::to_value(request()).expect("request JSON");
    open["future"] = serde_json::json!(true);
    assert!(parse_api_request_json(&serde_json::to_vec(&open).expect("bytes")).is_err());
}

fn request() -> GitHubApiRequest {
    GitHubApiRequest {
        schema: API_REQUEST_SCHEMA.into(),
        request_id: "github-request-1".into(),
        operation_id: "github-operation-1".into(),
        connection_ref: "github-connection-1".into(),
        owner: "example-org".into(),
        repository: "example-api".into(),
        action: GitHubAction::ObserveRepositoryMetadata,
    }
}
