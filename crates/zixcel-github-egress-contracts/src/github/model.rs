//! Closed, metadata-only GitHub authorization, command and receipt documents.
use serde::{Deserialize, Serialize};

pub const GITHUB_AUTHORIZATION_SCHEMA: &str = "crowsi://provider-egress/github-authorization/v1";
pub const GITHUB_COMMAND_SCHEMA: &str = "crowsi://provider-egress/github-command/v1";
pub const GITHUB_VERIFICATION_SCHEMA: &str =
    "crowsi://provider-egress/github-verification-receipt/v1";
pub const GITHUB_RECEIPT_SCHEMA: &str = "crowsi://provider-egress/github-receipt/v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubAuthorizationV1 {
    pub schema: String,
    pub authorization_id: String,
    pub issuer: String,
    pub audience: String,
    pub grant_digest_sha256: String,
    pub invocation_digest_sha256: String,
    pub api_request_digest_sha256: String,
    pub connection_ref_digest_sha256: String,
    pub pairwise_subject: String,
    pub device_ref: String,
    pub workload_id: String,
    pub grant_id: String,
    pub proof_key_ref: String,
    pub permission_resource: String,
    pub permission_operation: String,
    pub issued_at_epoch_s: u64,
    pub expires_at_epoch_s: u64,
    pub key_id: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubEgressCommandV1 {
    pub schema: String,
    pub mode: GitHubEgressMode,
    pub authorization: GitHubAuthorizationV1,
    pub grant_json: String,
    pub invocation_json: String,
    pub api_request_json: String,
    pub now_epoch_s: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitHubEgressMode {
    Verify,
    Execute,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubVerificationReceiptV1 {
    pub schema: String,
    pub authorization_id: String,
    pub authorization_digest_sha256: String,
    pub grant_digest_sha256: String,
    pub invocation_digest_sha256: String,
    pub api_request_digest_sha256: String,
    pub expires_at_epoch_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubEgressReceiptV1 {
    pub schema: String,
    pub authorization_id: String,
    pub authorization_digest_sha256: String,
    pub api_request_digest_sha256: String,
    pub provider_request_id: String,
    pub remote_reference: String,
    pub remote_digest_sha256: Option<String>,
    pub completed_at_epoch_s: u64,
}
