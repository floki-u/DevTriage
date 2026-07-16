pub mod fingerprint;
pub mod graph;
pub mod model;
pub mod normalize;
pub mod pack;
pub mod redact;
pub mod universal;

pub use model::{AnalysisDepth, EvidenceKind, IssueContext};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
