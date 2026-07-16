use crate::model::{Evidence, Transformation};
use regex::{Captures, Regex};
use std::sync::OnceLock;

fn assignment_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?i)\b(token|secret|password|passwd|api[_-]?key)\s*[:=]\s*([^\s,;]+)").unwrap()
    })
}

fn jwt_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b").unwrap()
    })
}

fn email_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap()
    })
}

pub fn redact_evidence(evidence: &mut [Evidence]) -> Vec<Transformation> {
    let mut assignment_count = 0usize;
    let mut jwt_count = 0usize;
    let mut email_count = 0usize;

    for item in evidence {
        let value = assignment_regex().replace_all(&item.value, |caps: &Captures<'_>| {
            assignment_count += 1;
            format!("{}=[REDACTED:CREDENTIAL]", &caps[1])
        });
        let value = jwt_regex().replace_all(&value, |_: &Captures<'_>| {
            jwt_count += 1;
            "[REDACTED:JWT]"
        });
        let value = email_regex().replace_all(&value, |_: &Captures<'_>| {
            email_count += 1;
            "[REDACTED:EMAIL]"
        });
        if value != item.value {
            item.value = value.into_owned();
            item.sensitivity = crate::model::Sensitivity::Secret;
        }
    }

    [
        ("credential_redacted", "Credential-like values redacted", assignment_count),
        ("jwt_redacted", "JWT-like values redacted", jwt_count),
        ("email_redacted", "Email addresses redacted", email_count),
    ]
    .into_iter()
    .filter(|(_, _, count)| *count > 0)
    .map(|(kind, detail, count)| Transformation {
        kind: kind.into(),
        detail: detail.into(),
        count,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EvidenceKind, Provenance, Sensitivity};

    #[test]
    fn redacts_assignments_jwts_and_emails_without_revealing_values() {
        let mut evidence = vec![Evidence {
            id: 1,
            kind: EvidenceKind::Error,
            value: "token=supersecret user=a@example.com jwt=eyJabc.def.ghi".into(),
            confidence: 90,
            sensitivity: Sensitivity::Public,
            provenance: vec![Provenance {
                source_id: "clipboard".into(),
                range: None,
                capability_id: "test".into(),
            }],
            related: vec![],
        }];
        let transformations = redact_evidence(&mut evidence);
        assert!(!evidence[0].value.contains("supersecret"));
        assert!(!evidence[0].value.contains("a@example.com"));
        assert!(!evidence[0].value.contains("eyJabc"));
        assert_eq!(transformations.iter().map(|item| item.count).sum::<usize>(), 3);
        assert_eq!(evidence[0].sensitivity, Sensitivity::Secret);
    }
}
