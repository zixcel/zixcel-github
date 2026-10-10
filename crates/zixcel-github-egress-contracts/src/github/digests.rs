//! Domain-separated hashing; signatures are excluded only from the unsigned authorization.
use super::GitHubAuthorizationV1;
use super::primitives::hex_digest;
use sha2::{Digest, Sha256};

/// Computes the domain-separated digest of the authorization without its signature.
///
/// # Errors
///
/// Returns an error if the closed authorization cannot be serialized.
pub fn authorization_digest(value: &GitHubAuthorizationV1) -> Result<String, &'static str> {
    let mut unsigned = value.clone();
    unsigned.signature_hex.clear();
    let bytes = serde_json::to_vec(&unsigned).map_err(|_| "github-authorization-invalid")?;
    let mut hash = Sha256::new();
    hash.update(b"crowsi:provider-egress:github-authorization:v1\0");
    hash.update(bytes);
    Ok(format!("sha256:{}", hex_digest(hash.finalize())))
}

#[must_use]
pub fn github_document_digest(value: &str) -> String {
    format!("sha256:{}", hex_digest(Sha256::digest(value.as_bytes())))
}

#[must_use]
pub fn github_connection_ref_digest(value: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(b"crowsi:provider-egress:github-connection-ref:v1\0");
    hash.update(value.as_bytes());
    format!("sha256:{}", hex_digest(hash.finalize()))
}
