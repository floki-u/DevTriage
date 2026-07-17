pub mod commands;
pub mod domain;
pub mod projects;
pub mod recent;

use devtriage_core::IssueContext;
use std::path::Path;
use std::sync::Mutex;

pub use domain::{CandidateReason, ProjectCandidate, RecentProject};
pub use projects::rank_projects;
pub use recent::{RecentProjectError, RecentProjectStore};

/// Application-local state. Raw logs and their analysis are deliberately never persisted.
pub struct DesktopState {
    raw_input: Mutex<Option<String>>,
    current_context: Mutex<Option<IssueContext>>,
    recent_projects: RecentProjectStore,
}

impl DesktopState {
    pub fn new(recent_projects: RecentProjectStore) -> Self {
        Self {
            raw_input: Mutex::new(None),
            current_context: Mutex::new(None),
            recent_projects,
        }
    }

    pub fn with_recent_projects_at(directory: impl AsRef<Path>) -> Self {
        Self::new(RecentProjectStore::at(directory))
    }

    #[cfg(test)]
    pub fn for_test() -> Self {
        Self::with_recent_projects_at(
            std::env::temp_dir().join(format!("devtriage-desktop-test-{}", std::process::id())),
        )
    }
}

pub fn run() {}
