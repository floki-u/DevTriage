use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub confirmed_at: i64,
}

impl RecentProject {
    pub fn new(path: impl AsRef<Path>, confirmed_at: i64) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().into_owned(),
            confirmed_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCandidate {
    pub path: String,
    pub score: u16,
    pub reason: CandidateReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateReason {
    AbsolutePath,
    RecentRelativePath,
    RecentProject,
}
