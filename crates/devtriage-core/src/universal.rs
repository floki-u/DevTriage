use crate::model::{
    AnalysisDepth, EvidenceDraft, EvidenceKind, Provenance, Sensitivity, SourceRange,
    Transformation,
};
use crate::normalize::NormalizedInput;
use crate::pack::{CapabilityPack, PackDescriptor, PackError, PackOutput};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub struct UniversalPack;

fn location_regex() -> &'static Regex {
    static LOCATION: OnceLock<Regex> = OnceLock::new();
    LOCATION.get_or_init(|| {
        Regex::new(r"(?P<path>(?:[A-Za-z]:)?[A-Za-z0-9_./\\-]+\.[A-Za-z0-9]+):(?P<line>\d+)(?::(?P<column>\d+))?")
            .unwrap()
    })
}

fn looks_like_error(line: &str) -> bool {
    let lower = line.to_lowercase();
    if lower.trim() == "showing all errors only" {
        return false;
    }

    [
        "error",
        "exception",
        "panic",
        "fatal",
        "failed",
        "couldn't find",
        "cannot find",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn looks_like_sensitive_candidate(line: &str) -> bool {
    let lower = line.to_lowercase();
    [
        "token=",
        "token:",
        "secret=",
        "secret:",
        "password=",
        "password:",
        "passwd=",
        "api_key=",
        "api-key=",
        "jwt=",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || line.contains('@')
}

impl CapabilityPack for UniversalPack {
    fn descriptor(&self) -> PackDescriptor {
        PackDescriptor {
            id: "official.universal",
            depth: AnalysisDepth::Generic,
        }
    }

    fn detect(&self, _: &NormalizedInput) -> u8 {
        1
    }

    fn analyze(&self, input: &NormalizedInput) -> Result<PackOutput, PackError> {
        let mut evidence = Vec::new();
        let mut counts = BTreeMap::<&str, usize>::new();

        for line in input
            .text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            *counts.entry(line).or_default() += 1;
        }

        if let Some((offset, line)) = input
            .text
            .lines()
            .scan(0usize, |offset, line| {
                let start = *offset;
                *offset += line.len() + 1;
                Some((start, line))
            })
            .find(|(_, line)| looks_like_error(line))
        {
            evidence.push(EvidenceDraft {
                kind: EvidenceKind::Error,
                value: line.trim().into(),
                confidence: 70,
                sensitivity: Sensitivity::Public,
                provenance: Provenance {
                    source_id: "normalized_input".into(),
                    range: Some(SourceRange {
                        start: offset,
                        end: offset + line.len(),
                    }),
                    capability_id: "official.universal".into(),
                },
            });
        }

        if !evidence.iter().any(|item| item.kind == EvidenceKind::Error)
            && let Some(line) = input
                .text
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
        {
            evidence.push(EvidenceDraft {
                kind: EvidenceKind::LogExcerpt,
                value: line.into(),
                confidence: 30,
                sensitivity: Sensitivity::Public,
                provenance: Provenance {
                    source_id: "normalized_input".into(),
                    range: None,
                    capability_id: "official.universal".into(),
                },
            });
        }

        for (offset, line) in input.text.lines().scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        }) {
            if looks_like_sensitive_candidate(line) {
                evidence.push(EvidenceDraft {
                    kind: EvidenceKind::LogExcerpt,
                    value: line.trim().into(),
                    confidence: 60,
                    sensitivity: Sensitivity::Sensitive,
                    provenance: Provenance {
                        source_id: "normalized_input".into(),
                        range: Some(SourceRange {
                            start: offset,
                            end: offset + line.len(),
                        }),
                        capability_id: "official.universal".into(),
                    },
                });
            }
        }

        for captures in location_regex().captures_iter(&input.text) {
            let whole = captures.get(0).unwrap();
            evidence.push(EvidenceDraft {
                kind: EvidenceKind::StackFrame,
                value: whole.as_str().into(),
                confidence: 80,
                sensitivity: Sensitivity::Sensitive,
                provenance: Provenance {
                    source_id: "normalized_input".into(),
                    range: Some(SourceRange {
                        start: whole.start(),
                        end: whole.end(),
                    }),
                    capability_id: "official.universal".into(),
                },
            });
        }

        let duplicates = counts.values().map(|count| count.saturating_sub(1)).sum();
        let transformations = if duplicates > 0 {
            vec![Transformation {
                kind: "duplicate_lines_collapsed".into(),
                detail: "Repeated identical lines were represented once".into(),
                count: duplicates,
            }]
        } else {
            Vec::new()
        };

        Ok(PackOutput {
            evidence,
            transformations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_primary_error_and_source_location() {
        let input = NormalizedInput {
            text: "TypeError: cannot read value\n  at render (src/UserList.tsx:42:7)".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert!(
            output
                .evidence
                .iter()
                .any(|e| e.kind == EvidenceKind::Error)
        );
        assert!(
            output
                .evidence
                .iter()
                .any(|e| e.kind == EvidenceKind::StackFrame && e.value == "src/UserList.tsx:42:7")
        );
    }

    #[test]
    fn ignores_xcode_error_filter_labels_when_selecting_the_primary_error() {
        let input = NormalizedInput {
            text: "Showing All Errors Only\nXcode couldn't find any iOS App Development provisioning profiles matching 'Netease.NeteaseMusicTests'.".into(),
            transformations: vec![],
        };

        let output = UniversalPack.analyze(&input).unwrap();
        let primary_error = output
            .evidence
            .iter()
            .find(|evidence| evidence.kind == EvidenceKind::Error)
            .expect("a provisioning failure should be classified as an error");

        assert_eq!(
            primary_error.value,
            "Xcode couldn't find any iOS App Development provisioning profiles matching 'Netease.NeteaseMusicTests'."
        );
    }

    #[test]
    fn reports_duplicate_lines() {
        let input = NormalizedInput {
            text: "warning: retry\nwarning: retry\nwarning: retry".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert_eq!(output.transformations[0].kind, "duplicate_lines_collapsed");
        assert_eq!(output.transformations[0].count, 2);
    }

    #[test]
    fn unknown_text_still_produces_a_generic_excerpt() {
        let input = NormalizedInput {
            text: "unexpected state while starting worker".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert!(
            output
                .evidence
                .iter()
                .any(|e| e.kind == EvidenceKind::LogExcerpt)
        );
    }

    #[test]
    fn retains_sensitive_candidate_lines_for_later_redaction() {
        let input = NormalizedInput {
            text: "Fatal error\ntoken=do-not-leak-this".into(),
            transformations: vec![],
        };
        let output = UniversalPack.analyze(&input).unwrap();
        assert!(
            output
                .evidence
                .iter()
                .any(|e| e.value == "token=do-not-leak-this")
        );
    }
}
