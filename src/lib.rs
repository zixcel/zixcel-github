#![forbid(unsafe_code)]
#![doc = "Policy-neutral typed GitHub API wrapper boundary."]

mod action;
mod action_validation;
mod artifacts;
mod boundary;
mod config;
mod execution;
mod execution_model;
mod plan;
mod reports;
mod secret_boundary;
#[cfg(test)]
mod unit_tests;

pub use action::{GitHubAction, GitHubRemoteExpectation, PermissionRequirement};
pub use artifacts::{
    GitHubSnapshotArtifact, GitHubSnapshotFile, GitHubTextArtifact, GitHubWorkflowInputsArtifact,
    parse_snapshot_artifact, parse_text_artifact, parse_workflow_inputs_artifact,
};
pub use boundary::ConnectorError;
pub use config::{ConnectorConfig, parse_config};
pub use execution::{
    execute_api_request, execute_configured_request, parse_api_request_json, validate_api_request,
};
pub use execution_model::{
    GitHubApiReceipt, GitHubApiRequest, GitHubBackend, GitHubBackendReceipt,
    RequestValidationReport,
};
pub use plan::{ConnectorPlan, PlanStep, build_plan};
pub use reports::{
    CapabilityReport, DoctorReport, ValidationReport, capabilities, doctor, validation_report,
};

pub const CONNECTOR: &str = "zixcel-github";
pub const PROVIDER: &str = "github";
pub const CONFIG_SCHEMA: &str = "zixcel://github/connector-config/v1";
pub const PLAN_SCHEMA: &str = "zixcel://contracts/connector-plan/v1";
pub const API_REQUEST_SCHEMA: &str = "zixcel://github/api-request/v1";
pub const API_RECEIPT_SCHEMA: &str = "zixcel://github/api-receipt/v1";
