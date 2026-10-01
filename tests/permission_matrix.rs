mod support;

use support::{RecordingBackend, request};
use zixcel_github::{
    GitHubAction, GitHubRemoteExpectation, PermissionRequirement, execute_api_request,
};

#[test]
fn every_action_has_one_exact_permission_pair() {
    for (action, expected) in cases() {
        assert_eq!(action.required_permission(), expected);
        let request = request(action);
        let mut backend = RecordingBackend::default();
        execute_api_request(&request, &mut backend).expect("valid API request");
        assert_eq!(backend.calls(), 1, "{}", expected.resource);
    }
}

fn cases() -> Vec<(GitHubAction, PermissionRequirement)> {
    vec![
        case(
            GitHubAction::ObserveRepositoryMetadata,
            "github-repository-metadata",
            "inspect-metadata",
        ),
        case(
            GitHubAction::ListIssues { maximum_items: 10 },
            "github-issue-observation",
            "list-records",
        ),
        case(
            GitHubAction::ListPullRequests { maximum_items: 10 },
            "github-pull-request-observation",
            "list-records",
        ),
        case(
            GitHubAction::ListReleases { maximum_items: 10 },
            "github-release-observation",
            "list-records",
        ),
        case(
            GitHubAction::ListWorkflowRuns { maximum_items: 10 },
            "github-workflow-run-observation",
            "list-records",
        ),
        case(
            GitHubAction::ListWorkflowArtifacts {
                workflow_run_id: "123".into(),
                maximum_items: 10,
            },
            "github-actions-artifact-observation",
            "list-records",
        ),
        case(
            GitHubAction::DownloadWorkflowArtifact {
                workflow_run_id: "123".into(),
                artifact_id: "456".into(),
                expected_digest_sha256: None,
            },
            "github-actions-artifact",
            "download-artifact",
        ),
        case(
            GitHubAction::CreatePrivateRepository {
                default_branch: "main".into(),
            },
            "github-private-repository",
            "create-private-resource",
        ),
        case(
            GitHubAction::PushRepositorySnapshot {
                snapshot_ref: "artifact/source-snapshot-1".into(),
                snapshot_digest_sha256: "a".repeat(64),
                expected_remote: GitHubRemoteExpectation::Absent,
            },
            "github-repository-content",
            "publish-artifact",
        ),
        case(
            GitHubAction::DeleteRepository {
                expected_repository_id: "12345".into(),
                backup_digest_sha256: "a".repeat(64),
            },
            "github-repository-administration",
            "delete-resource",
        ),
        case(
            GitHubAction::CreateIssue {
                title: "Issue title".into(),
                body_artifact_ref: "artifact/issue-body-1".into(),
            },
            "github-issue-creation",
            "create-work-item",
        ),
        case(
            GitHubAction::CreatePullRequest {
                base: "main".into(),
                head: "feature".into(),
                title: "Pull request title".into(),
                body_artifact_ref: "artifact/pull-request-body-1".into(),
            },
            "github-pull-request-creation",
            "create-review-request",
        ),
        case(
            GitHubAction::DispatchWorkflow {
                workflow_ref: "ci.yml".into(),
                git_ref: "main".into(),
                inputs_artifact_ref: None,
            },
            "github-workflow",
            "dispatch-operation",
        ),
    ]
}

const fn case(
    action: GitHubAction,
    resource: &'static str,
    operation: &'static str,
) -> (GitHubAction, PermissionRequirement) {
    (
        action,
        PermissionRequirement {
            resource,
            operation,
        },
    )
}
