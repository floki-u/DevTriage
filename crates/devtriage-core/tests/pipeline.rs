use devtriage_core::{OutputBudget, Pipeline};

#[test]
fn compiles_a_redacted_traceable_context() {
    let input = include_str!("fixtures/js-error.log");
    let context = Pipeline::default().analyze(input, OutputBudget::Standard);
    assert_eq!(context.schema_version, 1);
    assert!(context.output.text.contains("TypeError"));
    assert!(context.output.text.contains("src/pages/UserList.tsx:42:7"));
    assert!(
        context
            .transformations
            .iter()
            .any(|item| item.kind == "duplicate_lines_collapsed")
    );
    assert_eq!(context.fingerprint.len(), 64);
}

#[test]
fn secrets_never_reach_compiled_output() {
    let input = include_str!("fixtures/mixed-secrets.log");
    let context = Pipeline::default().analyze(input, OutputBudget::Standard);
    assert!(!context.output.text.contains("do-not-leak-this"));
    assert!(!context.output.text.contains("developer@example.com"));
    assert!(context.output.text.contains("[REDACTED:CREDENTIAL]"));
}
