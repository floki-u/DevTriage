use devtriage_desktop::{RecentProject, RecentProjectStore};

#[test]
fn confirm_overwrites_path_and_keeps_only_timestamp_and_path() {
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("app");
    std::fs::create_dir(&project).unwrap();
    let store = RecentProjectStore::at(directory.path());

    store.confirm(&project, 10).unwrap();

    assert_eq!(
        store.load().unwrap(),
        vec![RecentProject::new(project.canonicalize().unwrap(), 10)]
    );
    assert_eq!(
        std::fs::read_to_string(directory.path().join("recent-projects.json")).unwrap(),
        serde_json::to_string(&store.load().unwrap()).unwrap()
    );
}

#[test]
fn confirm_replaces_existing_path_and_retains_ten_most_recent_records() {
    let directory = tempfile::tempdir().unwrap();
    let store = RecentProjectStore::at(directory.path());
    let projects = (0..11)
        .map(|index| {
            let path = directory.path().join(format!("project-{index}"));
            std::fs::create_dir(&path).unwrap();
            path
        })
        .collect::<Vec<_>>();

    for (index, path) in projects.iter().enumerate() {
        store.confirm(path, index as i64).unwrap();
    }
    store.confirm(&projects[1], 20).unwrap();

    let records = store.load().unwrap();
    assert_eq!(records.len(), 10);
    assert_eq!(
        records[0].path,
        projects[1].canonicalize().unwrap().to_string_lossy()
    );
    assert_eq!(records[0].confirmed_at, 20);
    assert!(
        !records
            .iter()
            .any(|record| record.path == projects[0].canonicalize().unwrap().to_string_lossy())
    );
}
