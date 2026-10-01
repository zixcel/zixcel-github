use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{CONNECTOR, ConnectorConfig, ConnectorError, PLAN_SCHEMA, PROVIDER};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanStep {
    pub sequence: u32,
    pub action: &'static str,
    pub target: String,
    pub effect: &'static str,
    pub network_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectorPlan {
    pub schema: &'static str,
    pub plan_id: String,
    pub request_id: String,
    pub provider: &'static str,
    pub connector: &'static str,
    pub mode: &'static str,
    pub steps: Vec<PlanStep>,
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Serialize)]
struct PlanSeed<'a> {
    request_id: &'a str,
    provider: &'static str,
    connector: &'static str,
    steps: &'a [PlanStep],
    extensions: &'a BTreeMap<String, Value>,
}

pub fn build_plan(config: &ConnectorConfig) -> Result<ConnectorPlan, ConnectorError> {
    config.validate()?;
    let config = config.normalized();
    let request_id = format!("request-{}", config.config_id);
    let mut steps = vec![PlanStep {
        sequence: 1,
        action: "validate-config",
        target: config.config_id.clone(),
        effect: "none",
        network_required: false,
    }];
    for (index, repository) in config.repositories.iter().enumerate() {
        steps.push(PlanStep {
            sequence: u32::try_from(index + 2)
                .map_err(|_| ConnectorError::new("repositories", "too many repositories"))?,
            action: "prepare-api-request",
            target: format!("{}/{}", config.organization, repository),
            effect: "normalize",
            network_required: false,
        });
    }
    let extensions = BTreeMap::from([
        ("connection_ref".to_owned(), json!(config.connection_ref)),
        ("allowed_actions".to_owned(), json!(config.allowed_actions)),
        (
            "safety".to_owned(),
            json!({
                "planning_only": true,
                "network_calls_performed": false,
                "credentials_resolved": false,
                "codex_credential_reuse": false
            }),
        ),
    ]);
    let seed = PlanSeed {
        request_id: &request_id,
        provider: PROVIDER,
        connector: CONNECTOR,
        steps: &steps,
        extensions: &extensions,
    };
    let bytes = serde_json::to_vec(&seed)
        .map_err(|_| ConnectorError::new("plan", "could not serialize canonical plan"))?;
    let digest = Sha256::digest(bytes);
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(ConnectorPlan {
        schema: PLAN_SCHEMA,
        plan_id: format!("plan-{PROVIDER}-{}", &hex[..16]),
        request_id,
        provider: PROVIDER,
        connector: CONNECTOR,
        mode: "propose",
        steps,
        extensions,
    })
}
