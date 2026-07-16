use devtriage_desktop::{CandidateReason, RecentProject, rank_projects};

#[test]
fn absolute_log_path_ranks_matching_recent_project_first() {
    let recent = vec![RecentProject::new("/work/app", 10)];
    let candidates = rank_projects("at run (/work/app/src/main.ts:4:2)", &recent);

    assert_eq!(candidates[0].path, "/work/app");
    assert_eq!(candidates[0].reason, CandidateReason::AbsolutePath);
}

#[test]
fn relative_log_paths_rank_before_plain_recent_projects() {
    let recent = vec![
        RecentProject::new("/work/older", 10),
        RecentProject::new("/work/newer", 20),
    ];
    let candidates = rank_projects("at run (src/main.ts:4:2)", &recent);

    assert_eq!(candidates[0].reason, CandidateReason::RecentRelativePath);
    assert_eq!(candidates[0].path, "/work/newer");
    assert!(candidates[0].score > candidates[1].score);
}

#[test]
fn ties_sort_by_path() {
    let recent = vec![
        RecentProject::new("/work/zebra", 10),
        RecentProject::new("/work/alpha", 10),
    ];
    let candidates = rank_projects("no path here", &recent);

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| &candidate.path)
            .collect::<Vec<_>>(),
        vec!["/work/alpha", "/work/zebra"]
    );
}
