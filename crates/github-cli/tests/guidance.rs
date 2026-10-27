use std::process::Command;
#[test]
fn all_operations_have_owned_secret_free_guidance() {
    let (m, c) = zixcel_github_cli::guided().unwrap();
    assert_eq!(m.protocol, "zix-provider/v2");
    for op in &m.commands {
        assert!(
            c.entries
                .iter()
                .any(|g| g.operation == op.id && g.selectors.is_empty())
        );
    }
    let o = Command::new(env!("CARGO_BIN_EXE_zix-github"))
        .arg("--guidance")
        .env_clear()
        .output()
        .unwrap();
    assert!(o.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap(),
        serde_json::to_value(c).unwrap()
    );
}
