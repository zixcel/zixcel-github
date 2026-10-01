use crate::boundary::{bounded_text, digest, reference};
use crate::{ConnectorError, GitHubAction, GitHubRemoteExpectation};

impl GitHubAction {
    pub(crate) fn validate(&self) -> Result<(), ConnectorError> {
        use GitHubAction::*;
        match self {
            ObserveRepositoryMetadata => Ok(()),
            ListIssues { maximum_items }
            | ListPullRequests { maximum_items }
            | ListReleases { maximum_items }
            | ListWorkflowRuns { maximum_items } => validate_maximum(*maximum_items),
            ListWorkflowArtifacts {
                workflow_run_id,
                maximum_items,
            } => {
                bounded_text("workflow_run_id", workflow_run_id, 128)?;
                validate_maximum(*maximum_items)
            }
            DownloadWorkflowArtifact {
                workflow_run_id,
                artifact_id,
                expected_digest_sha256,
            } => {
                bounded_text("workflow_run_id", workflow_run_id, 128)?;
                bounded_text("artifact_id", artifact_id, 128)?;
                if let Some(value) = expected_digest_sha256 {
                    digest("expected_digest_sha256", value)?;
                }
                Ok(())
            }
            CreatePrivateRepository { default_branch } => {
                if !crate::artifacts::git_branch(default_branch) {
                    return Err(ConnectorError::new("default_branch", "invalid Git branch"));
                }
                Ok(())
            }
            PushRepositorySnapshot {
                snapshot_ref,
                snapshot_digest_sha256,
                expected_remote,
            } => {
                reference("snapshot_ref", snapshot_ref)?;
                digest("snapshot_digest_sha256", snapshot_digest_sha256)?;
                if let GitHubRemoteExpectation::Exact { commit_sha256 } = expected_remote {
                    digest("commit_sha256", commit_sha256)?;
                }
                Ok(())
            }
            DeleteRepository {
                expected_repository_id,
                backup_digest_sha256,
            } => {
                if expected_repository_id.is_empty()
                    || expected_repository_id.len() > 20
                    || !expected_repository_id.bytes().all(|v| v.is_ascii_digit())
                    || expected_repository_id.starts_with('0')
                {
                    return Err(ConnectorError::new(
                        "expected_repository_id",
                        "expected a positive repository ID",
                    ));
                }
                digest("backup_digest_sha256", backup_digest_sha256)
            }
            CreateIssue {
                title,
                body_artifact_ref,
            } => validate_body(title, body_artifact_ref),
            CreatePullRequest {
                base,
                head,
                title,
                body_artifact_ref,
            } => {
                bounded_text("base", base, 128)?;
                bounded_text("head", head, 128)?;
                validate_body(title, body_artifact_ref)
            }
            DispatchWorkflow {
                workflow_ref,
                git_ref,
                inputs_artifact_ref,
            } => {
                bounded_text("workflow_ref", workflow_ref, 256)?;
                bounded_text("git_ref", git_ref, 256)?;
                if let Some(value) = inputs_artifact_ref {
                    reference("inputs_artifact_ref", value)?;
                }
                Ok(())
            }
        }
    }
}

fn validate_maximum(value: u16) -> Result<(), ConnectorError> {
    if (1..=256).contains(&value) {
        Ok(())
    } else {
        Err(ConnectorError::new("maximum_items", "must be in 1..=256"))
    }
}

fn validate_body(title: &str, body: &str) -> Result<(), ConnectorError> {
    bounded_text("title", title, 256)?;
    reference("body_artifact_ref", body)
}
