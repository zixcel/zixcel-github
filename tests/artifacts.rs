use zixcel_github::{parse_snapshot_artifact, parse_text_artifact, parse_workflow_inputs_artifact};

#[test]
fn closed_text_and_inputs_artifacts_accept_only_bounded_declared_fields() {
    let text = br#"{"schema":"zixcel://github/text-artifact/v1","text":"body"}"#;
    assert_eq!(parse_text_artifact(text).expect("text").text, "body");
    assert!(
        parse_text_artifact(
            br#"{"schema":"zixcel://github/text-artifact/v1","text":"body","extra":true}"#,
        )
        .is_err()
    );
    let inputs =
        br#"{"schema":"zixcel://github/workflow-inputs-artifact/v1","inputs":{"release":"v1"}}"#;
    assert_eq!(
        parse_workflow_inputs_artifact(inputs)
            .expect("inputs")
            .inputs["release"],
        "v1"
    );
    assert!(
        parse_workflow_inputs_artifact(
            br#"{"schema":"zixcel://github/workflow-inputs-artifact/v1","inputs":{" bad ":"v1"}}"#,
        )
        .is_err()
    );
}

#[test]
fn snapshot_rejects_duplicate_traversal_and_invalid_base64() {
    let valid = br#"{"schema":"zixcel://github/source-snapshot-artifact/v1","branch":"main","message":"release","files":[{"path":"README.md","mode":"100644","content_base64":"aGVsbG8="}]}"#;
    let parsed = parse_snapshot_artifact(valid).expect("snapshot");
    assert_eq!(parsed.files[0].path, "README.md");
    for invalid in [
        br#"{"schema":"zixcel://github/source-snapshot-artifact/v1","branch":"main","message":"release","files":[{"path":"../README.md","mode":"100644","content_base64":"aGVsbG8="}]}"#.as_slice(),
        br#"{"schema":"zixcel://github/source-snapshot-artifact/v1","branch":"main","message":"release","files":[{"path":"README.md","mode":"100644","content_base64":"aGVsbG8="},{"path":"README.md","mode":"100755","content_base64":"aGVsbG8="}]}"#.as_slice(),
        br#"{"schema":"zixcel://github/source-snapshot-artifact/v1","branch":"main","message":"release","files":[{"path":"README.md","mode":"100644","content_base64":"***"}]}"#.as_slice(),
    ] {
        assert!(parse_snapshot_artifact(invalid).is_err());
    }
}
