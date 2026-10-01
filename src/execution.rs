use crate::boundary::{digest, github_name, identifier, reference};
use crate::{
    API_RECEIPT_SCHEMA, API_REQUEST_SCHEMA, ConnectorError, GitHubApiReceipt, GitHubApiRequest,
    GitHubBackend, GitHubBackendReceipt, RequestValidationReport,
};

const MAX_DOCUMENT_BYTES: usize = 1_048_576;

impl GitHubApiRequest {
    pub fn validate(&self) -> Result<(), ConnectorError> {
        if self.schema != API_REQUEST_SCHEMA {
            return Err(ConnectorError::new(
                "schema",
                "unexpected action request schema",
            ));
        }
        identifier("request_id", &self.request_id)?;
        identifier("operation_id", &self.operation_id)?;
        reference("connection_ref", &self.connection_ref)?;
        github_name("owner", &self.owner)?;
        github_name("repository", &self.repository)?;
        self.action.validate()
    }
}

pub fn parse_api_request_json(source: &[u8]) -> Result<GitHubApiRequest, ConnectorError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(ConnectorError::new("request", "document exceeds 1 MiB"));
    }
    let value: GitHubApiRequest = serde_json::from_slice(source)
        .map_err(|_| ConnectorError::new("request", "invalid closed JSON"))?;
    value.validate()?;
    Ok(value)
}

pub fn validate_api_request(
    request: &GitHubApiRequest,
) -> Result<RequestValidationReport, ConnectorError> {
    request.validate()?;
    let required = request.action.required_permission();
    Ok(RequestValidationReport {
        schema: "zixcel://github/request-validation-result/v1",
        request_id: request.request_id.clone(),
        operation_id: request.operation_id.clone(),
        valid: true,
        required_permission: required,
    })
}

pub fn execute_api_request<B: GitHubBackend>(
    request: &GitHubApiRequest,
    backend: &mut B,
) -> Result<GitHubApiReceipt, ConnectorError> {
    let report = validate_api_request(request)?;
    let value = backend.perform(request)?;
    validate_backend_receipt(&value)?;
    Ok(GitHubApiReceipt {
        schema: API_RECEIPT_SCHEMA,
        request_id: request.request_id.clone(),
        operation_id: request.operation_id.clone(),
        owner: request.owner.clone(),
        repository: request.repository.clone(),
        permission: report.required_permission,
        provider_request_id: value.provider_request_id,
        remote_reference: value.remote_reference,
        remote_digest_sha256: value.remote_digest_sha256,
    })
}

fn validate_backend_receipt(value: &GitHubBackendReceipt) -> Result<(), ConnectorError> {
    identifier("provider_request_id", &value.provider_request_id)?;
    reference("remote_reference", &value.remote_reference)?;
    if let Some(value) = &value.remote_digest_sha256 {
        digest("remote_digest_sha256", value)?;
    }
    Ok(())
}

/// Execute only after the explicit local connection/target/action ceiling passes.
pub fn execute_configured_request<B: GitHubBackend>(
    config: &crate::ConnectorConfig,
    request: &GitHubApiRequest,
    backend: &mut B,
) -> Result<GitHubApiReceipt, ConnectorError> {
    config.authorize_request(request)?;
    execute_api_request(request, backend)
}
