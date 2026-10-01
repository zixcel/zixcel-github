use serde::Serialize;

use crate::{CONNECTOR, ConnectorConfig, ConnectorError, PROVIDER};

const CAPABILITIES: &[&str] = &[
    "issue-observe",
    "pull-request-observe",
    "release-observe",
    "repository-metadata-observe",
    "workflow-run-observe",
    "workflow-artifact-observe",
    "workflow-artifact-download",
    "private-repository-create",
    "repository-snapshot-push",
    "repository-delete",
    "issue-create",
    "pull-request-create",
    "workflow-dispatch",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    pub schema: &'static str,
    pub connector: &'static str,
    pub status: &'static str,
    pub network_client_linked: bool,
    pub secret_resolution_enabled: bool,
    pub execution_enabled: bool,
    pub embedded_authorization_policy: bool,
    pub caller_authorization_required: bool,
    pub codex_credential_reuse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityReport {
    pub schema: &'static str,
    pub connector: &'static str,
    pub provider: &'static str,
    pub planning_only: bool,
    pub capabilities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationReport {
    pub schema: &'static str,
    pub connector: &'static str,
    pub config_id: String,
    pub valid: bool,
}

#[must_use]
pub const fn doctor() -> DoctorReport {
    DoctorReport {
        schema: "zixcel://doctor/v1",
        connector: CONNECTOR,
        status: "healthy",
        network_client_linked: false,
        secret_resolution_enabled: false,
        execution_enabled: true,
        embedded_authorization_policy: false,
        caller_authorization_required: true,
        codex_credential_reuse: false,
    }
}

#[must_use]
pub fn capabilities() -> CapabilityReport {
    CapabilityReport {
        schema: "zixcel://capabilities/v1",
        connector: CONNECTOR,
        provider: PROVIDER,
        planning_only: false,
        capabilities: CAPABILITIES.to_vec(),
    }
}

pub fn validation_report(config: &ConnectorConfig) -> Result<ValidationReport, ConnectorError> {
    config.validate()?;
    Ok(ValidationReport {
        schema: "zixcel://validation-result/v1",
        connector: CONNECTOR,
        config_id: config.config_id.clone(),
        valid: true,
    })
}
