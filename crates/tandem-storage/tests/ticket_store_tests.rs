use std::fs;

use tandem_storage::discover_repo_root;

#[test]
#[allow(clippy::disallowed_methods)]
fn discover_repo_root_finds_git_dir() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(repo_root.path().join(".git")).expect("create .git dir");

    let nested = repo_root.path().join("crates").join("tandem-storage");
    fs::create_dir_all(&nested).expect("create nested dir");

    let discovered = discover_repo_root(&nested).expect("discover repo root");

    assert_eq!(discovered, repo_root.path());
}

#[test]
#[allow(clippy::disallowed_methods)]
fn discover_repo_root_errors_when_no_repo_markers() {
    let start = tempfile::tempdir().expect("tempdir");

    let error = discover_repo_root(start.path()).expect_err("discover should fail");

    assert_eq!(
        error.to_string(),
        "no repository markers found (.tndm or .git)"
    );
}
