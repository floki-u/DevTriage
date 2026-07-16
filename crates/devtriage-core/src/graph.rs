use crate::model::{Evidence, EvidenceDraft};
use std::collections::BTreeMap;

pub fn merge(drafts: Vec<EvidenceDraft>) -> Vec<Evidence> {
    let mut grouped: BTreeMap<(crate::model::EvidenceKind, String), Evidence> = BTreeMap::new();

    for draft in drafts {
        let normalized = draft
            .value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let key = (draft.kind, normalized.to_lowercase());
        let next_id = grouped.len() as u64 + 1;
        let entry = grouped.entry(key).or_insert_with(|| Evidence {
            id: next_id,
            kind: draft.kind,
            value: normalized,
            confidence: draft.confidence,
            sensitivity: draft.sensitivity,
            provenance: Vec::new(),
            related: Vec::new(),
        });
        entry.confidence = entry.confidence.max(draft.confidence);
        entry.sensitivity = entry.sensitivity.max(draft.sensitivity);
        if !entry.provenance.contains(&draft.provenance) {
            entry.provenance.push(draft.provenance);
        }
    }

    grouped
        .into_values()
        .enumerate()
        .map(|(index, mut item)| {
            item.id = index as u64 + 1;
            item
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EvidenceKind, Provenance, Sensitivity};

    fn draft(capability_id: &str, confidence: u8) -> EvidenceDraft {
        EvidenceDraft {
            kind: EvidenceKind::Error,
            value: " TypeError: boom ".into(),
            confidence,
            sensitivity: Sensitivity::Public,
            provenance: Provenance {
                source_id: "clipboard".into(),
                range: None,
                capability_id: capability_id.into(),
            },
        }
    }

    #[test]
    fn merges_equivalent_evidence_and_keeps_both_sources() {
        let graph = merge(vec![draft("generic", 70), draft("javascript", 95)]);
        assert_eq!(graph.len(), 1);
        assert_eq!(graph[0].id, 1);
        assert_eq!(graph[0].confidence, 95);
        assert_eq!(graph[0].provenance.len(), 2);
        assert_eq!(graph[0].value, "TypeError: boom");
    }
}
