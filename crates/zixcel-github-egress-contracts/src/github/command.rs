//! Validate exact GitHub request bindings before any provider effect.
use super::primitives::{digest, id, reference};
use super::{
    GITHUB_AUTHORIZATION_SCHEMA, GITHUB_COMMAND_SCHEMA, GitHubEgressCommandV1,
    github_document_digest,
};

/// Validates the bounded GitHub egress command and every content digest.
///
/// # Errors
///
/// Returns an error when the command is malformed, expired, or not exactly
/// correlated with its embedded documents.
pub fn validate_github_command(value: &GitHubEgressCommandV1) -> Result<(), &'static str> {
    let auth = &value.authorization;
    if value.schema != GITHUB_COMMAND_SCHEMA
        || auth.schema != GITHUB_AUTHORIZATION_SCHEMA
        || !id(&auth.authorization_id, 128)
        || !reference(&auth.issuer, 256)
        || auth.audience != "zixcel-github-transport"
        || !digest(&auth.grant_digest_sha256)
        || !digest(&auth.invocation_digest_sha256)
        || !digest(&auth.api_request_digest_sha256)
        || !digest(&auth.connection_ref_digest_sha256)
        || !reference(&auth.pairwise_subject, 128)
        || !reference(&auth.device_ref, 128)
        || !reference(&auth.workload_id, 256)
        || !id(&auth.grant_id, 128)
        || !reference(&auth.proof_key_ref, 128)
        || !id(&auth.permission_resource, 128)
        || !id(&auth.permission_operation, 128)
        || auth.issued_at_epoch_s > value.now_epoch_s
        || value.now_epoch_s >= auth.expires_at_epoch_s
        || auth.expires_at_epoch_s <= auth.issued_at_epoch_s
        || auth.expires_at_epoch_s - auth.issued_at_epoch_s > 300
        || !id(&auth.key_id, 128)
        || auth.signature_hex.len() != 128
        || !auth
            .signature_hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || value.grant_json.is_empty()
        || value.grant_json.len() > 262_144
        || value.invocation_json.is_empty()
        || value.invocation_json.len() > 262_144
        || value.api_request_json.is_empty()
        || value.api_request_json.len() > 1_048_576
        || github_document_digest(&value.grant_json) != auth.grant_digest_sha256
        || github_document_digest(&value.invocation_json) != auth.invocation_digest_sha256
        || github_document_digest(&value.api_request_json) != auth.api_request_digest_sha256
    {
        return Err("github-egress-command-invalid");
    }
    Ok(())
}
