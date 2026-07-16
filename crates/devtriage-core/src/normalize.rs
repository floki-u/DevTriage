use crate::model::Transformation;
use regex::Regex;
use std::sync::OnceLock;

pub const MAX_INPUT_BYTES: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInput {
    pub text: String,
    pub transformations: Vec<Transformation>,
}

fn ansi_regex() -> &'static Regex {
    static ANSI: OnceLock<Regex> = OnceLock::new();
    ANSI.get_or_init(|| Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap())
}

pub fn normalize(input: &str) -> NormalizedInput {
    let ansi_count = ansi_regex().find_iter(input).count();
    let without_ansi = ansi_regex().replace_all(input, "");
    let mut text = without_ansi.replace("\r\n", "\n").replace('\r', "\n");
    let mut transformations = Vec::new();

    if ansi_count > 0 {
        transformations.push(Transformation {
            kind: "ansi_removed".into(),
            detail: "Removed terminal control sequences".into(),
            count: ansi_count,
        });
    }

    if text.len() > MAX_INPUT_BYTES {
        let mut boundary = MAX_INPUT_BYTES;
        while !text.is_char_boundary(boundary) {
            boundary -= 1;
        }
        text.truncate(boundary);
        transformations.push(Transformation {
            kind: "input_truncated".into(),
            detail: format!("Input limited to {MAX_INPUT_BYTES} bytes"),
            count: 1,
        });
    }

    NormalizedInput {
        text,
        transformations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_and_normalizes_newlines() {
        let result = normalize("\u{1b}[31mError\u{1b}[0m\r\nnext\rline");
        assert_eq!(result.text, "Error\nnext\nline");
        assert_eq!(result.transformations[0].kind, "ansi_removed");
    }

    #[test]
    fn truncates_on_utf8_boundary() {
        let input = "界".repeat(MAX_INPUT_BYTES);
        let result = normalize(&input);
        assert!(result.text.len() <= MAX_INPUT_BYTES);
        assert!(result.text.is_char_boundary(result.text.len()));
        assert!(
            result
                .transformations
                .iter()
                .any(|item| item.kind == "input_truncated")
        );
    }
}
