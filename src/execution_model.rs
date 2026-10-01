use serde::{Deserialize, Serialize};

use crate::{ConnectorError, GitHubAction, PermissionRequirement};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubApiRequest {
    pub schema: String,
    pub request_id: String,
    pub operation_id: String,
    pub connection_ref: String,
    pub owner: String,
    pub repository: String,
    pub action: GitHubAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestValidationReport {
    pub schema: &'static str,
    pub request_id: String,
    pub operation_id: String,
    pub valid: bool,
    pub required_permission: PermissionRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubBackendReceipt {
    pub provider_request_id: String,
    pub remote_reference: String,
    pub remote_digest_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitHubApiReceipt {
    pub schema: &'static str,
    pub request_id: String,
    pub operation_id: String,
    pub owner: String,
    pub repository: String,
    pub permission: PermissionRequirement,
    pub provider_request_id: String,
    pub remote_reference: String,
    pub remote_digest_sha256: Option<String>,
}

pub trait GitHubBackend {
    fn perform(
        &mut self,
        request: &GitHubApiRequest,
    ) -> Result<GitHubBackendReceipt, ConnectorError>;
}
