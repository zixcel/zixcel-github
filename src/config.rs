use serde::{Deserialize, Serialize};

use crate::boundary::{github_name, identifier, reference};
use crate::secret_boundary::reject_secrets;
use crate::{CONFIG_SCHEMA, ConnectorError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    pub schema: String,
    pub config_id: String,
    pub connection_ref: String,
    pub organization: String,
    pub repositories: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
}

impl ConnectorConfig {
    pub fn validate(&self) -> Result<(), ConnectorError> {
        if self.schema != CONFIG_SCHEMA {
            return Err(ConnectorError::new(
                "schema",
                "expected zixcel://github/connector-config/v1",
            ));
        }
        identifier("config_id", &self.config_id)?;
        reference("connection_ref", &self.connection_ref)?;
        github_name("organization", &self.organization)?;
        if self.repositories.is_empty() || self.repositories.len() > 256 {
            return Err(ConnectorError::new(
                "repositories",
                "must contain 1..=256 repositories",
            ));
        }
        for repository in &self.repositories {
            github_name("repositories", repository)?;
        }
        const WRITES: &[&str] = &[
            "create-private-repository",
            "push-repository-snapshot",
            "delete-repository",
            "create-issue",
            "create-pull-request",
            "dispatch-workflow",
        ];
        if self
            .allowed_actions
            .iter()
            .any(|action| !WRITES.contains(&action.as_str()))
            || self.allowed_actions.len() > WRITES.len()
        {
            return Err(ConnectorError::new(
                "allowed_actions",
                "unknown or excessive write capabilities",
            ));
        }
        Ok(())
    }

    /// Local capability ceiling; does not replace a signed caller grant or provider permissions.
    pub fn authorize_request(
        &self,
        request: &crate::GitHubApiRequest,
    ) -> Result<(), ConnectorError> {
        self.validate()?;
        crate::validate_api_request(request)?;
        if self.connection_ref != request.connection_ref
            || !self.organization.eq_ignore_ascii_case(&request.owner)
            || !self
                .repositories
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&request.repository))
        {
            return Err(ConnectorError::new(
                "target",
                "request is outside the configured connection and repositories",
            ));
        }
        if !request.action.is_read_only()
            && !self
                .allowed_actions
                .iter()
                .any(|action| action == request.action.kind())
        {
            return Err(ConnectorError::new(
                "action",
                "write action is not enabled by local configuration",
            ));
        }
        Ok(())
    }

    pub(crate) fn normalized(&self) -> Self {
        let mut value = self.clone();
        value.allowed_actions.sort();
        value.allowed_actions.dedup();
        value.repositories.sort_by_key(|item| item.to_lowercase());
        value
            .repositories
            .dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        value
    }
}

pub fn parse_config(source: &str) -> Result<ConnectorConfig, ConnectorError> {
    reject_secrets(source)?;
    let config: ConnectorConfig = toml::from_str(source).map_err(|_| {
        ConnectorError::new(
            "config",
            "invalid TOML or fields do not match the GitHub v1 schema",
        )
    })?;
    config.validate()?;
    Ok(config)
}
