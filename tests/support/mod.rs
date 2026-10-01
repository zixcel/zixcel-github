use zixcel_github::{
    API_REQUEST_SCHEMA, ConnectorError, GitHubAction, GitHubApiRequest, GitHubBackend,
    GitHubBackendReceipt,
};

pub const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub fn request(action: GitHubAction) -> GitHubApiRequest {
    GitHubApiRequest {
        schema: API_REQUEST_SCHEMA.into(),
        request_id: "github-request-1".into(),
        operation_id: "github-operation-1".into(),
        connection_ref: "github-connection-1".into(),
        owner: "example-org".into(),
        repository: "example-api".into(),
        action,
    }
}

#[derive(Default)]
pub struct RecordingBackend {
    calls: usize,
}

impl RecordingBackend {
    pub const fn calls(&self) -> usize {
        self.calls
    }
}

impl GitHubBackend for RecordingBackend {
    fn perform(
        &mut self,
        _request: &GitHubApiRequest,
    ) -> Result<GitHubBackendReceipt, ConnectorError> {
        self.calls += 1;
        Ok(GitHubBackendReceipt {
            provider_request_id: "github-provider-1".into(),
            remote_reference: "github/example-org/example-api".into(),
            remote_digest_sha256: Some(DIGEST.into()),
        })
    }
}
