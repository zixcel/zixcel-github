use sha2::{Digest, Sha256};

use crate::{
    GitHubAuthorizationV1, GitHubEgressCommandV1, GitHubEgressMode, GitHubVerificationReceiptV1,
    authorization_digest, github_connection_ref_digest, validate_github_command,
    validate_github_verification_receipt,
};

#[test]
fn exact_documents_are_bound_and_substitution_is_rejected() {
    let mut command = command();
    assert!(validate_github_command(&command).is_ok());
    let receipt = GitHubVerificationReceiptV1 {
        schema: "crowsi://provider-egress/github-verification-receipt/v1".into(),
        authorization_id: "authorization-1".into(),
        authorization_digest_sha256: authorization_digest(&command.authorization)
            .expect("authorization digest"),
        grant_digest_sha256: digest(&command.grant_json),
        invocation_digest_sha256: digest(&command.invocation_json),
        api_request_digest_sha256: digest(&command.api_request_json),
        expires_at_epoch_s: 200,
    };
    assert!(validate_github_verification_receipt(&receipt, &command).is_ok());
    command.api_request_json.push(' ');
    assert!(validate_github_command(&command).is_err());
}

fn command() -> GitHubEgressCommandV1 {
    let grant_json = r#"{"grant":"one"}"#.to_owned();
    let invocation_json = r#"{"invocation":"one"}"#.to_owned();
    let api_request_json = r#"{"request":"one"}"#.to_owned();
    GitHubEgressCommandV1 {
        schema: "crowsi://provider-egress/github-command/v1".into(),
        mode: GitHubEgressMode::Verify,
        authorization: GitHubAuthorizationV1 {
            schema: "crowsi://provider-egress/github-authorization/v1".into(),
            authorization_id: "authorization-1".into(),
            issuer: "crowsi-pa".into(),
            audience: "zixcel-github-transport".into(),
            grant_digest_sha256: digest(&grant_json),
            invocation_digest_sha256: digest(&invocation_json),
            api_request_digest_sha256: digest(&api_request_json),
            connection_ref_digest_sha256: github_connection_ref_digest("connection-1"),
            pairwise_subject: "subject-pairwise-1".into(),
            device_ref: "device-1".into(),
            workload_id: "spiffe://crowsi.local/hat/github-operator".into(),
            grant_id: "grant-1".into(),
            proof_key_ref: "proof-key-1".into(),
            permission_resource: "github-repository-content".into(),
            permission_operation: "publish-artifact".into(),
            issued_at_epoch_s: 100,
            expires_at_epoch_s: 200,
            key_id: "crowsi-pa-key-1".into(),
            signature_hex: "ab".repeat(64),
        },
        grant_json,
        invocation_json,
        api_request_json,
        now_epoch_s: 150,
    }
}

fn digest(value: &str) -> String {
    (Sha256::digest(value.as_bytes()))
        .iter()
        .fold(String::from("sha256:"), |mut encoded, byte| {
            use std::fmt::Write as _;
            write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
            encoded
        })
}
