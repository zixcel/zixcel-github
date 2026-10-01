use zixcel_github::{build_plan, doctor, parse_config};

const CONFIG: &str = r#"
schema = "zixcel://github/connector-config/v1"
config_id = "example-catalog"
connection_ref = "connection:github:owner"
organization = "example-org"
repositories = ["example-api", "example-web"]
"#;

#[test]
fn equivalent_repository_order_is_canonical() {
    let alternate = CONFIG.replace(
        "[\"example-api\", \"example-web\"]",
        "[\"example-web\", \"example-api\"]",
    );
    let first = build_plan(&parse_config(CONFIG).expect("config")).expect("plan");
    let second = build_plan(&parse_config(&alternate).expect("config")).expect("plan");
    assert_eq!(first, second);
}

#[test]
fn rejects_open_insecure_or_secret_bearing_input() {
    assert!(parse_config(&CONFIG.replace("connection:github:owner", "\nunsafe")).is_err());
    assert!(parse_config(&format!("{CONFIG}\ntoken = \"no\"\n")).is_err());
    assert!(parse_config(&format!("{CONFIG}\nfuture = true\n")).is_err());
    assert!(parse_config(&" ".repeat(1_048_577)).is_err());
}

#[test]
fn doctor_exposes_a_policy_neutral_api_wrapper() {
    let report = doctor();
    assert!(!report.network_client_linked);
    assert!(!report.secret_resolution_enabled);
    assert!(report.execution_enabled);
    assert!(!report.codex_credential_reuse);
    assert!(!report.embedded_authorization_policy);
    assert!(report.caller_authorization_required);
}
