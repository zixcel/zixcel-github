use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PermissionRequirement {
    pub resource: &'static str,
    pub operation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GitHubRemoteExpectation {
    Absent,
    Exact { commit_sha256: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GitHubAction {
    ObserveRepositoryMetadata,
    ListIssues {
        maximum_items: u16,
    },
    ListPullRequests {
        maximum_items: u16,
    },
    ListReleases {
        maximum_items: u16,
    },
    ListWorkflowRuns {
        maximum_items: u16,
    },
    ListWorkflowArtifacts {
        workflow_run_id: String,
        maximum_items: u16,
    },
    DownloadWorkflowArtifact {
        workflow_run_id: String,
        artifact_id: String,
        expected_digest_sha256: Option<String>,
    },
    CreatePrivateRepository {
        default_branch: String,
    },
    PushRepositorySnapshot {
        snapshot_ref: String,
        snapshot_digest_sha256: String,
        expected_remote: GitHubRemoteExpectation,
    },
    DeleteRepository {
        expected_repository_id: String,
        backup_digest_sha256: String,
    },
    CreateIssue {
        title: String,
        body_artifact_ref: String,
    },
    CreatePullRequest {
        base: String,
        head: String,
        title: String,
        body_artifact_ref: String,
    },
    DispatchWorkflow {
        workflow_ref: String,
        git_ref: String,
        inputs_artifact_ref: Option<String>,
    },
}

impl GitHubAction {
    #[must_use]
    pub const fn required_permission(&self) -> PermissionRequirement {
        use GitHubAction::*;
        match self {
            ObserveRepositoryMetadata => {
                permission("github-repository-metadata", "inspect-metadata")
            }
            ListIssues { .. } => permission("github-issue-observation", "list-records"),
            ListPullRequests { .. } => {
                permission("github-pull-request-observation", "list-records")
            }
            ListReleases { .. } => permission("github-release-observation", "list-records"),
            ListWorkflowRuns { .. } => {
                permission("github-workflow-run-observation", "list-records")
            }
            ListWorkflowArtifacts { .. } => {
                permission("github-actions-artifact-observation", "list-records")
            }
            DownloadWorkflowArtifact { .. } => {
                permission("github-actions-artifact", "download-artifact")
            }
            CreatePrivateRepository { .. } => {
                permission("github-private-repository", "create-private-resource")
            }
            PushRepositorySnapshot { .. } => {
                permission("github-repository-content", "publish-artifact")
            }
            DeleteRepository { .. } => {
                permission("github-repository-administration", "delete-resource")
            }
            CreateIssue { .. } => permission("github-issue-creation", "create-work-item"),
            CreatePullRequest { .. } => {
                permission("github-pull-request-creation", "create-review-request")
            }
            DispatchWorkflow { .. } => permission("github-workflow", "dispatch-operation"),
        }
    }
}

const fn permission(resource: &'static str, operation: &'static str) -> PermissionRequirement {
    PermissionRequirement {
        resource,
        operation,
    }
}

impl GitHubAction {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ObserveRepositoryMetadata => "observe-repository-metadata",
            Self::ListIssues { .. } => "list-issues",
            Self::ListPullRequests { .. } => "list-pull-requests",
            Self::ListReleases { .. } => "list-releases",
            Self::ListWorkflowRuns { .. } => "list-workflow-runs",
            Self::ListWorkflowArtifacts { .. } => "list-workflow-artifacts",
            Self::DownloadWorkflowArtifact { .. } => "download-workflow-artifact",
            Self::CreatePrivateRepository { .. } => "create-private-repository",
            Self::PushRepositorySnapshot { .. } => "push-repository-snapshot",
            Self::DeleteRepository { .. } => "delete-repository",
            Self::CreateIssue { .. } => "create-issue",
            Self::CreatePullRequest { .. } => "create-pull-request",
            Self::DispatchWorkflow { .. } => "dispatch-workflow",
        }
    }
    pub fn is_read_only(&self) -> bool {
        matches!(
            self,
            Self::ObserveRepositoryMetadata
                | Self::ListIssues { .. }
                | Self::ListPullRequests { .. }
                | Self::ListReleases { .. }
                | Self::ListWorkflowRuns { .. }
                | Self::ListWorkflowArtifacts { .. }
                | Self::DownloadWorkflowArtifact { .. }
        )
    }
}
