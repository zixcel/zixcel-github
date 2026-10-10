//! Correlate independent verification and provider execution receipts.
use super::primitives::{digest, id, reference};
use super::{
    GITHUB_RECEIPT_SCHEMA, GITHUB_VERIFICATION_SCHEMA, GitHubEgressCommandV1,
    GitHubEgressReceiptV1, GitHubVerificationReceiptV1, authorization_digest,
};

/// Correlates one verification receipt with the exact authorization command.
///
/// # Errors
///
/// Returns an error for any identity, digest, or expiry mismatch.
pub fn validate_github_verification_receipt(
    value: &GitHubVerificationReceiptV1,
    command: &GitHubEgressCommandV1,
) -> Result<(), &'static str> {
    if value.schema != GITHUB_VERIFICATION_SCHEMA
        || value.authorization_id != command.authorization.authorization_id
        || value.authorization_digest_sha256 != authorization_digest(&command.authorization)?
        || value.grant_digest_sha256 != command.authorization.grant_digest_sha256
        || value.invocation_digest_sha256 != command.authorization.invocation_digest_sha256
        || value.api_request_digest_sha256 != command.authorization.api_request_digest_sha256
        || value.expires_at_epoch_s != command.authorization.expires_at_epoch_s
    {
        return Err("github-verification-receipt-invalid");
    }
    Ok(())
}

/// Correlates a metadata-only provider receipt with the exact command.
///
/// # Errors
///
/// Returns an error for an invalid provider result or authorization mismatch.
pub fn validate_github_egress_receipt(
    value: &GitHubEgressReceiptV1,
    command: &GitHubEgressCommandV1,
) -> Result<(), &'static str> {
    if value.schema != GITHUB_RECEIPT_SCHEMA
        || value.authorization_id != command.authorization.authorization_id
        || value.authorization_digest_sha256 != authorization_digest(&command.authorization)?
        || value.api_request_digest_sha256 != command.authorization.api_request_digest_sha256
        || !id(&value.provider_request_id, 128)
        || !reference(&value.remote_reference, 512)
        || value
            .remote_digest_sha256
            .as_ref()
            .is_some_and(|item| !digest(item))
        || value.completed_at_epoch_s < command.authorization.issued_at_epoch_s
        || value.completed_at_epoch_s > command.now_epoch_s
    {
        return Err("github-egress-receipt-invalid");
    }
    Ok(())
}
