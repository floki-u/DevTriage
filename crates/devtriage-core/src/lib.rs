pub mod model;

pub use model::{AnalysisDepth, EvidenceKind, IssueContext};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
