#[allow(dead_code)]
mod support;
use support::request;
use zixcel_github::{GitHubAction, build_plan, parse_config};
fn policy(writes: &str) -> zixcel_github::ConnectorConfig {
    parse_config(&format!(
        r#"schema = "zixcel://github/connector-config/v1"
config_id = "local"
connection_ref = "{}"
organization = "{}"
repositories = ["{}"]
{}
"#,
        request(GitHubAction::ObserveRepositoryMetadata).connection_ref,
        request(GitHubAction::ObserveRepositoryMetadata).owner,
        request(GitHubAction::ObserveRepositoryMetadata).repository,
        writes
    ))
    .expect("policy")
}
#[test]
fn writes_are_disabled_by_default_and_bound_to_target_and_connection() {
    let read = request(GitHubAction::ObserveRepositoryMetadata);
    assert!(policy("").authorize_request(&read).is_ok());
    let write = request(GitHubAction::CreatePrivateRepository {
        default_branch: "main".into(),
    });
    assert!(policy("").authorize_request(&write).is_err());
    let enabled = policy("allowed_actions = [\"create-private-repository\"]");
    assert!(enabled.authorize_request(&write).is_ok());
    let mut foreign = write.clone();
    foreign.owner = "other".into();
    assert!(enabled.authorize_request(&foreign).is_err());
    foreign = write.clone();
    foreign.connection_ref = "connection/other".into();
    assert!(enabled.authorize_request(&foreign).is_err());
    assert_ne!(
        build_plan(&enabled).unwrap().plan_id,
        build_plan(&policy("")).unwrap().plan_id
    );
}
#[test]
fn deletion_requires_a_separate_permission_and_valid_backup_binding() {
    let deletion = request(GitHubAction::DeleteRepository {
        expected_repository_id: "12345".into(),
        backup_digest_sha256: "a".repeat(64),
    });
    assert!(
        policy("allowed_actions = [\"create-private-repository\", \"push-repository-snapshot\"]")
            .authorize_request(&deletion)
            .is_err()
    );
    assert!(
        policy("allowed_actions = [\"delete-repository\"]")
            .authorize_request(&deletion)
            .is_ok()
    );
    assert_eq!(
        deletion.action.required_permission().resource,
        "github-repository-administration"
    );
    let mut invalid = deletion;
    invalid.action = GitHubAction::DeleteRepository {
        expected_repository_id: "0".into(),
        backup_digest_sha256: "a".repeat(64),
    };
    assert!(
        policy("allowed_actions = [\"delete-repository\"]")
            .authorize_request(&invalid)
            .is_err()
    );
}

#[test]
fn unsafe_creation_branch_is_rejected_before_any_effect() {
    let request = request(GitHubAction::CreatePrivateRepository {
        default_branch: "../unsafe".into(),
    });
    assert!(
        policy("allowed_actions = [\"create-private-repository\"]")
            .authorize_request(&request)
            .is_err()
    );
}
