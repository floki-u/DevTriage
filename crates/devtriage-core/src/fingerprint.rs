use crate::model::{Evidence, EvidenceKind};

pub fn fingerprint(evidence: &[Evidence]) -> String {
    let canonical = evidence
        .iter()
        .filter(|item| {
            matches!(
                item.kind,
                EvidenceKind::Error | EvidenceKind::StackFrame | EvidenceKind::LogExcerpt
            )
        })
        .map(|item| format!("{:?}:{}", item.kind, item.value.to_lowercase()))
        .collect::<Vec<_>>()
        .join("\n");
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Provenance, Sensitivity};

    fn item(value: &str) -> Evidence {
        Evidence {
            id: 1,
            kind: EvidenceKind::Error,
            value: value.into(),
            confidence: 90,
            sensitivity: Sensitivity::Public,
            provenance: vec![Provenance {
                source_id: "clipboard".into(),
                range: None,
                capability_id: "test".into(),
            }],
            related: vec![],
        }
    }

    #[test]
    fn fingerprint_is_stable_and_value_sensitive() {
        assert_eq!(fingerprint(&[item("Boom")]), fingerprint(&[item("boom")]));
        assert_ne!(fingerprint(&[item("Boom")]), fingerprint(&[item("Other")]));
    }
}
