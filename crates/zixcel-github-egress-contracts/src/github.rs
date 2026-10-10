//! GitHub-specific contracts; account credentials and HTTP effects belong to callers.
mod command;
mod digests;
mod model;
mod primitives;
mod receipts;
pub use command::validate_github_command;
pub use digests::{authorization_digest, github_connection_ref_digest, github_document_digest};
use model::{
    GITHUB_AUTHORIZATION_SCHEMA, GITHUB_COMMAND_SCHEMA, GITHUB_RECEIPT_SCHEMA,
    GITHUB_VERIFICATION_SCHEMA,
};
pub use model::{
    GitHubAuthorizationV1, GitHubEgressCommandV1, GitHubEgressMode, GitHubEgressReceiptV1,
    GitHubVerificationReceiptV1,
};
pub use receipts::{validate_github_egress_receipt, validate_github_verification_receipt};
