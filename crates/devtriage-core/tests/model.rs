use devtriage_core::model::{AnalysisDepth, EvidenceKind, SCHEMA_VERSION};

#[test]
fn model_has_stable_external_names() {
    assert_eq!(SCHEMA_VERSION, 1);
    assert_eq!(
        serde_json::to_string(&AnalysisDepth::Structured).unwrap(),
        "\"structured\""
    );
    assert!(EvidenceKind::Error < EvidenceKind::StackFrame);
}
