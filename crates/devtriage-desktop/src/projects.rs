use crate::domain::{CandidateReason, ProjectCandidate, RecentProject};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const ABSOLUTE_PATH_SCORE: u16 = 300;
const RELATIVE_PATH_SCORE: u16 = 200;
const RECENT_PROJECT_SCORE: u16 = 100;
const MAX_RECENCY_BONUS: u16 = 99;

fn absolute_path_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"/[^\x0D\n:()\[\]{}]+(?::\d+(?::\d+)?)?").unwrap())
}

fn relative_path_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?:^|[\s(\[])\.?\.?[A-Za-z0-9_.-]*(?:/[A-Za-z0-9_. -]+)+\.[A-Za-z0-9_-]+(?::\d+(?::\d+)?)?")
            .unwrap()
    })
}

fn is_path_within(path: &str, project: &str) -> bool {
    let project = project.trim_end_matches('/');
    project.is_empty()
        || path == project
        || path
            .strip_prefix(project)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Ranks only supplied recent records. It deliberately does not inspect candidate directories.
pub fn rank_projects(input: &str, recent: &[RecentProject]) -> Vec<ProjectCandidate> {
    let absolute_paths = absolute_path_regex()
        .find_iter(input)
        .map(|matched| matched.as_str().split(':').next().unwrap())
        .collect::<Vec<_>>();
    let has_relative_path = relative_path_regex().is_match(input);

    // Keep this function pure even if callers accidentally provide duplicate records.
    let mut unique_projects = BTreeMap::<&str, i64>::new();
    for project in recent {
        unique_projects
            .entry(&project.path)
            .and_modify(|confirmed_at| *confirmed_at = (*confirmed_at).max(project.confirmed_at))
            .or_insert(project.confirmed_at);
    }

    let mut projects = unique_projects.into_iter().collect::<Vec<_>>();
    projects.sort_by(|(left_path, left_time), (right_path, right_time)| {
        right_time
            .cmp(left_time)
            .then_with(|| left_path.cmp(right_path))
    });

    let mut candidates = projects
        .into_iter()
        .enumerate()
        .map(|(index, (path, _))| {
            let (score, reason) = if absolute_paths
                .iter()
                .any(|absolute_path| is_path_within(absolute_path, path))
            {
                (ABSOLUTE_PATH_SCORE, CandidateReason::AbsolutePath)
            } else if has_relative_path {
                (RELATIVE_PATH_SCORE, CandidateReason::RecentRelativePath)
            } else {
                (RECENT_PROJECT_SCORE, CandidateReason::RecentProject)
            };
            ProjectCandidate {
                path: path.to_owned(),
                score: score + MAX_RECENCY_BONUS.saturating_sub(index as u16),
                reason,
            }
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    candidates
}
