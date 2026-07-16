use crate::domain::RecentProject;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const FILE_NAME: &str = "recent-projects.json";
const MAX_RECENT_PROJECTS: usize = 10;

#[derive(Debug, Error)]
pub enum RecentProjectError {
    #[error("could not access recent projects: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse recent projects: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct RecentProjectStore {
    directory: PathBuf,
}

impl RecentProjectStore {
    pub fn at(directory: impl AsRef<Path>) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self) -> Result<Vec<RecentProject>, RecentProjectError> {
        let path = self.path();
        match fs::read_to_string(path) {
            Ok(contents) => Ok(serde_json::from_str(&contents)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }

    /// Canonicalizes the user-confirmed path before persisting only that path and its timestamp.
    pub fn confirm(
        &self,
        path: impl AsRef<Path>,
        confirmed_at: i64,
    ) -> Result<(), RecentProjectError> {
        let confirmed = fs::canonicalize(path)?;
        let confirmed = RecentProject::new(confirmed, confirmed_at);
        let mut projects = self.load()?;
        projects.retain(|project| project.path != confirmed.path);
        projects.push(confirmed);
        projects.sort_by(|left, right| {
            right
                .confirmed_at
                .cmp(&left.confirmed_at)
                .then_with(|| left.path.cmp(&right.path))
        });
        projects.truncate(MAX_RECENT_PROJECTS);
        fs::write(self.path(), serde_json::to_string(&projects)?)?;
        Ok(())
    }

    fn path(&self) -> PathBuf {
        self.directory.join(FILE_NAME)
    }
}
