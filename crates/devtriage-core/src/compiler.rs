use crate::model::{CompiledOutput, Evidence, EvidenceKind, Transformation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputBudget {
    Compact,
    Standard,
    Detailed,
}

impl OutputBudget {
    fn token_limit(self) -> usize {
        match self {
            Self::Compact => 500,
            Self::Standard => 1_500,
            Self::Detailed => 4_000,
        }
    }
}

fn priority(kind: EvidenceKind) -> u8 {
    match kind {
        EvidenceKind::Error => 0,
        EvidenceKind::StackFrame => 1,
        EvidenceKind::Runtime => 2,
        EvidenceKind::Project => 3,
        EvidenceKind::FilePath => 4,
        EvidenceKind::LogExcerpt => 5,
    }
}

fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

pub fn compile(
    evidence: &[Evidence],
    transformations: &[Transformation],
    budget: OutputBudget,
) -> CompiledOutput {
    let mut ordered = evidence.to_vec();
    ordered.sort_by_key(|item| {
        (
            priority(item.kind),
            std::cmp::Reverse(item.confidence),
            item.id,
        )
    });

    let request = "## Request\nIdentify the most likely cause using only the facts above, then recommend the next diagnostic or repair step.";
    let mut text = String::from("## Facts\n");
    let mut omitted = 0usize;
    let max_chars = budget.token_limit() * 4;

    for item in ordered {
        let line = format!("- {:?}: {}\n", item.kind, item.value);
        if text.len() + line.len() + request.len() + 2 <= max_chars {
            text.push_str(&line);
        } else {
            omitted += 1;
        }
    }

    if !transformations.is_empty() {
        let summary = transformations
            .iter()
            .map(|item| format!("{}={}", item.kind, item.count))
            .collect::<Vec<_>>()
            .join(", ");
        let line = format!("\n## Transformations\n{summary}\n");
        if text.len() + line.len() + request.len() + 2 <= max_chars {
            text.push_str(&line);
        }
    }

    text.push('\n');
    text.push_str(request);
    if text.len() > max_chars {
        text.truncate(max_chars);
    }

    CompiledOutput {
        estimated_tokens: estimate_tokens(&text),
        text,
        omitted_evidence: omitted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Provenance, Sensitivity};

    fn evidence(kind: EvidenceKind, value: &str) -> Evidence {
        Evidence {
            id: 1,
            kind,
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
    fn standard_output_separates_facts_from_request() {
        let output = compile(
            &[
                evidence(EvidenceKind::Error, "TypeError: boom"),
                evidence(EvidenceKind::StackFrame, "src/a.ts:4:2"),
            ],
            &[],
            OutputBudget::Standard,
        );
        assert!(output.text.contains("## Facts"));
        assert!(output.text.contains("## Request"));
        assert!(output.text.contains("most likely cause"));
    }

    #[test]
    fn compact_output_respects_estimated_budget() {
        let long = "x".repeat(10_000);
        let output = compile(
            &[evidence(EvidenceKind::Error, &long)],
            &[],
            OutputBudget::Compact,
        );
        assert!(output.estimated_tokens <= 500);
        assert_eq!(output.omitted_evidence, 1);
    }
}
