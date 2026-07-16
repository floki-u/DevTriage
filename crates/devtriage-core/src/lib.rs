pub mod model;
pub mod normalize;

pub use model::{AnalysisDepth, EvidenceKind, IssueContext};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
