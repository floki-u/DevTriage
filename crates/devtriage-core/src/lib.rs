pub mod model;
pub mod normalize;
pub mod pack;

pub use model::{AnalysisDepth, EvidenceKind, IssueContext};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
