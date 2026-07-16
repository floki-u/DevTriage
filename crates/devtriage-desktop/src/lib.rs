pub mod domain;
pub mod projects;
pub mod recent;

pub use domain::{CandidateReason, ProjectCandidate, RecentProject};
pub use projects::rank_projects;
pub use recent::{RecentProjectError, RecentProjectStore};

pub fn run() {}
