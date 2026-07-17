use crate::{DesktopState, ProjectCandidate, RecentProjectError, rank_projects};
use devtriage_core::{IssueContext, OutputBudget, Pipeline};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct AnalysisResponse {
    pub context: IssueContext,
    pub candidates: Vec<ProjectCandidate>,
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("analysis state is temporarily unavailable")]
    StateUnavailable,
    #[error("the selected path is not a directory")]
    NotDirectory,
    #[error("could not access the selected path")]
    SelectedPath,
    #[error("could not determine the current time")]
    Clock,
    #[error(transparent)]
    Recent(#[from] RecentProjectError),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, CommandError> {
    mutex.lock().map_err(|_| CommandError::StateUnavailable)
}

fn confirmed_at() -> Result<i64, CommandError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CommandError::Clock)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| CommandError::Clock)
}

/// Analyzes only the supplied text and retains its raw form and result in memory for this run.
#[tauri::command]
pub fn analyze_input(
    state: State<'_, DesktopState>,
    input: String,
    budget: OutputBudget,
) -> Result<AnalysisResponse, CommandError> {
    analyze_with_state(state.inner(), input, budget)
}

fn analyze_with_state(
    state: &DesktopState,
    input: String,
    budget: OutputBudget,
) -> Result<AnalysisResponse, CommandError> {
    let context = Pipeline::default().analyze(&input, budget);
    *lock(&state.raw_input)? = Some(input.clone());
    *lock(&state.current_context)? = Some(context.clone());
    let candidates = project_candidates_with_state(state, &input)?;

    Ok(AnalysisResponse {
        context,
        candidates,
    })
}

/// Ranks recent user-confirmed projects from the supplied log text without accessing projects.
#[tauri::command]
pub fn project_candidates(
    state: State<'_, DesktopState>,
    input: String,
) -> Result<Vec<ProjectCandidate>, CommandError> {
    project_candidates_with_state(state.inner(), &input)
}

fn project_candidates_with_state(
    state: &DesktopState,
    input: &str,
) -> Result<Vec<ProjectCandidate>, CommandError> {
    let recent = state.recent_projects.load()?;
    Ok(rank_projects(input, &recent))
}

/// Saves only a canonical directory path and confirmation time after explicit user selection.
#[tauri::command]
pub fn confirm_project(state: State<'_, DesktopState>, path: String) -> Result<(), CommandError> {
    confirm_project_with_state(state.inner(), path)
}

fn confirm_project_with_state(state: &DesktopState, path: String) -> Result<(), CommandError> {
    let path = fs::canonicalize(Path::new(&path)).map_err(|_| CommandError::SelectedPath)?;
    if !path.is_dir() {
        return Err(CommandError::NotDirectory);
    }
    state.recent_projects.confirm(path, confirmed_at()?)?;
    Ok(())
}

/// Rebuilds the selected output view from the in-memory raw input, if any.
#[tauri::command]
pub fn current_context(
    state: State<'_, DesktopState>,
    budget: OutputBudget,
) -> Result<Option<IssueContext>, CommandError> {
    current_context_with_state(state.inner(), budget)
}

fn current_context_with_state(
    state: &DesktopState,
    budget: OutputBudget,
) -> Result<Option<IssueContext>, CommandError> {
    let Some(input) = lock(&state.raw_input)?.clone() else {
        return Ok(None);
    };
    let context = Pipeline::default().analyze(&input, budget);
    *lock(&state.current_context)? = Some(context.clone());
    Ok(Some(context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RecentProjectStore;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn analyzed_secret_never_reaches_copy_ready_pipeline_output() {
        let directory = tempfile::tempdir().unwrap();
        let state = DesktopState::with_recent_projects_at(directory.path());
        let response = analyze_with_state(
            &state,
            "fatal token=hidden-value".into(),
            OutputBudget::Standard,
        )
        .unwrap();
        assert!(
            response
                .context
                .output
                .text
                .contains("[REDACTED:CREDENTIAL]")
        );
        assert!(!response.context.output.text.contains("hidden-value"));
    }

    #[test]
    fn explicit_confirmation_persists_canonical_directory_and_time() {
        let storage = tempfile::tempdir().unwrap();
        let project = storage.path().join("project");
        fs::create_dir(&project).unwrap();
        let state = DesktopState::with_recent_projects_at(storage.path());
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        confirm_project_with_state(&state, project.join(".").display().to_string()).unwrap();

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let records = RecentProjectStore::at(storage.path()).load().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].path,
            project.canonicalize().unwrap().to_string_lossy()
        );
        let confirmed_at: u64 = records[0].confirmed_at.try_into().unwrap();
        assert!(confirmed_at >= before);
        assert!(confirmed_at <= after);
    }

    #[test]
    fn reloaded_confirmation_is_a_candidate_for_later_analysis() {
        let storage = tempfile::tempdir().unwrap();
        let project = storage.path().join("project");
        fs::create_dir(&project).unwrap();
        let first_run = DesktopState::with_recent_projects_at(storage.path());
        confirm_project_with_state(&first_run, project.display().to_string()).unwrap();

        let later_run = DesktopState::with_recent_projects_at(storage.path());
        let response = analyze_with_state(
            &later_run,
            "error in src/main.rs:7".into(),
            OutputBudget::Compact,
        )
        .unwrap();

        assert_eq!(response.candidates.len(), 1);
        assert_eq!(
            response.candidates[0].path,
            project.canonicalize().unwrap().to_string_lossy()
        );
        assert_eq!(
            response.candidates[0].reason,
            crate::CandidateReason::RecentRelativePath
        );
    }

    #[test]
    fn current_context_is_empty_before_analysis() {
        let state = DesktopState::for_test();
        assert!(
            current_context_with_state(&state, OutputBudget::Compact)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn confirming_a_file_is_rejected() {
        let state = DesktopState::for_test();
        let file = tempfile::NamedTempFile::new().unwrap();
        let error =
            confirm_project_with_state(&state, file.path().display().to_string()).unwrap_err();
        assert!(matches!(error, CommandError::NotDirectory));
    }
}
