use std::fs;

use tandem_storage::{TandemConfig, load_config};

const DEFAULT_CONTENT_TEMPLATE: &str = "## Description\n\n## Design\n\n## Acceptance\n\n## Notes\n";

#[test]
fn load_config_returns_defaults_when_file_missing() {
    let repo_root = tempfile::tempdir().expect("tempdir");

    let config = load_config(repo_root.path()).expect("load default config");

    assert_eq!(
        config,
        TandemConfig {
            id_prefix: "TNDM".to_string(),
            content_template: DEFAULT_CONTENT_TEMPLATE.to_string(),
        }
    );
}

#[test]
fn load_config_reads_prefix_and_template_from_config_file() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    let config_dir = repo_root.path().join(".tndm");
    fs::create_dir_all(&config_dir).expect("create .tndm dir");

    fs::write(
        config_dir.join("config.toml"),
        r#"schema_version = 1

[id]
prefix = "PROJ"

[templates]
content = '''
## What

## Why

## Done
'''
"#,
    )
    .expect("write config.toml");

    let config = load_config(repo_root.path()).expect("load repo config");

    assert_eq!(
        config,
        TandemConfig {
            id_prefix: "PROJ".to_string(),
            content_template: "## What\n\n## Why\n\n## Done\n".to_string(),
        }
    );
}

#[test]
fn load_config_rejects_unknown_schema_version() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    let config_dir = repo_root.path().join(".tndm");
    fs::create_dir_all(&config_dir).expect("create .tndm dir");

    fs::write(
        config_dir.join("config.toml"),
        r#"schema_version = 2

[id]
prefix = "TNDM"
"#,
    )
    .expect("write config.toml");

    let error = load_config(repo_root.path()).expect_err("unknown schema version should fail");

    assert!(
        error.to_string().contains("schema_version"),
        "expected schema_version error, got: {error}"
    );
}
