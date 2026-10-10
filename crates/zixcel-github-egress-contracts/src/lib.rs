//! Closed Crowsi route contracts for external provider egress.

mod github;
#[cfg(test)]
mod github_tests;

pub use github::{
    GitHubAuthorizationV1, GitHubEgressCommandV1, GitHubEgressMode, GitHubEgressReceiptV1,
    GitHubVerificationReceiptV1, authorization_digest, github_connection_ref_digest,
    github_document_digest, validate_github_command, validate_github_egress_receipt,
    validate_github_verification_receipt,
};
