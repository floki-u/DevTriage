pub mod compiler;
pub mod fingerprint;
pub mod graph;
pub mod model;
pub mod normalize;
pub mod pack;
pub mod pipeline;
pub mod redact;
pub mod universal;

pub use compiler::OutputBudget;
pub use model::{AnalysisDepth, EvidenceKind, IssueContext};
pub use pipeline::Pipeline;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
