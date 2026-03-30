#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

use cargo_metadata::MetadataCommand;

const XTASK_USAGE: &str = "usage: cargo xtask <check-arch | sync-version [--check]>";
const WORKSPACE_CRATES: &[&str] = &[
    "tandem-core",
    "tandem-storage",
    "tandem-repo",
    "tandem-cli",
    "xtask",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(violations) => {
            for violation in violations {
                eprintln!("{violation}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Vec<String>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check-arch") if args.next().is_none() => check_arch(),
        Some("sync-version") => {
            let next = args.next();
            let check = next.as_deref() == Some("--check");
            if next.is_some() && !check {
                return Err(vec![XTASK_USAGE.to_owned()]);
            }
            if args.next().is_some() {
                return Err(vec![XTASK_USAGE.to_owned()]);
            }
            sync_version(check)
        }
        Some(_) => Err(vec![XTASK_USAGE.to_owned()]),
        None => Err(vec![XTASK_USAGE.to_owned()]),
    }
}

fn check_arch() -> Result<(), Vec<String>> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .map_err(|error| vec![format!("failed to load cargo metadata: {error}")])?;

    let package_deps = build_dependency_index(&metadata);
    let mut violations = Vec::new();

    for crate_name in WORKSPACE_CRATES {
        if !package_deps.contains_key(*crate_name) {
            violations.push(format!("missing workspace crate `{crate_name}`"));
        }
    }

    let forbidden_edges = [
        ("tandem-core", "tandem-storage"),
        ("tandem-core", "tandem-repo"),
        ("tandem-core", "tandem-cli"),
        ("tandem-core", "xtask"),
        ("tandem-storage", "tandem-repo"),
        ("tandem-storage", "tandem-cli"),
        ("tandem-storage", "xtask"),
        ("tandem-repo", "tandem-storage"),
        ("tandem-repo", "tandem-cli"),
        ("tandem-repo", "xtask"),
        ("tandem-cli", "xtask"),
        ("xtask", "tandem-core"),
        ("xtask", "tandem-storage"),
        ("xtask", "tandem-repo"),
        ("xtask", "tandem-cli"),
    ];

    for (from, to) in forbidden_edges {
        if has_dependency(&package_deps, from, to) {
            violations.push(format!("forbidden dependency edge `{from} -> {to}`"));
        }
    }

    let required_edges = [
        ("tandem-storage", "tandem-core"),
        ("tandem-repo", "tandem-core"),
        ("tandem-cli", "tandem-core"),
        ("tandem-cli", "tandem-storage"),
        ("tandem-cli", "tandem-repo"),
        ("tandem-cli", "clap"),
    ];

    for (from, to) in required_edges {
        if !has_dependency(&package_deps, from, to) {
            violations.push(format!(
                "required dependency edge `{from} -> {to}` is missing"
            ));
        }
    }

    for (from, deps) in &package_deps {
        if from != "tandem-cli" && deps.contains("clap") {
            violations.push(format!(
                "crate `{from}` depends on `clap`; only `tandem-cli` may depend on `clap`"
            ));
        }
    }

    if violations.is_empty() {
        println!("architecture checks passed");
        return Ok(());
    }

    Err(violations)
}

fn build_dependency_index(metadata: &cargo_metadata::Metadata) -> HashMap<String, HashSet<String>> {
    metadata
        .packages
        .iter()
        .map(|package| {
            let dependencies = package
                .dependencies
                .iter()
                .filter(|dependency| dependency.kind == cargo_metadata::DependencyKind::Normal)
                .map(|dependency| dependency.name.to_string())
                .collect::<HashSet<_>>();
            (package.name.to_string(), dependencies)
        })
        .collect()
}

fn has_dependency(index: &HashMap<String, HashSet<String>>, from: &str, to: &str) -> bool {
    index.get(from).is_some_and(|deps| deps.contains(to))
}

fn find_repo_root() -> Result<PathBuf, Vec<String>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| vec!["CARGO_MANIFEST_DIR not set".to_owned()])?;
    let root = PathBuf::from(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| vec!["cannot determine repo root".to_owned()])?
        .to_owned();
    Ok(root)
}

fn read_workspace_version(cargo_toml_path: &Path) -> Result<String, Vec<String>> {
    let content = std::fs::read_to_string(cargo_toml_path)
        .map_err(|e| vec![format!("failed to read {}: {e}", cargo_toml_path.display())])?;
    let doc: toml::Table = toml::from_str(&content).map_err(|e| {
        vec![format!(
            "failed to parse {}: {e}",
            cargo_toml_path.display()
        )]
    })?;
    let version = doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| vec!["missing workspace.package.version in Cargo.toml".to_owned()])?;
    Ok(version.to_owned())
}

fn sync_json_at_pointer(path: &Path, pointer: &str, version: &str) -> Result<(), Vec<String>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| vec![format!("failed to read {}: {e}", path.display())])?;
    let mut doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", path.display())])?;
    let target = doc.pointer_mut(pointer).ok_or_else(|| {
        vec![format!(
            "JSON pointer {pointer} not found in {}",
            path.display()
        )]
    })?;
    *target = serde_json::Value::String(version.to_owned());
    let output = serde_json::to_string_pretty(&doc)
        .map_err(|e| vec![format!("failed to serialize {}: {e}", path.display())])?;
    std::fs::write(path, format!("{output}\n"))
        .map_err(|e| vec![format!("failed to write {}: {e}", path.display())])?;
    Ok(())
}

fn check_json_at_pointer(path: &Path, pointer: &str, version: &str) -> Result<bool, Vec<String>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| vec![format!("failed to read {}: {e}", path.display())])?;
    let doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", path.display())])?;
    Ok(doc.pointer(pointer).and_then(|v| v.as_str()) == Some(version))
}

fn sync_version(check: bool) -> Result<(), Vec<String>> {
    let root = find_repo_root()?;
    let version = read_workspace_version(&root.join("Cargo.toml"))?;

    let targets = [
        (
            "plugin",
            root.join("plugin/tndm/.claude-plugin/plugin.json"),
            "/version",
        ),
        (
            "marketplace",
            root.join(".claude-plugin/marketplace.json"),
            "/plugins/0/version",
        ),
    ];

    let mut mismatches = Vec::new();

    for (label, path, pointer) in &targets {
        match check_json_at_pointer(path, pointer, &version) {
            Ok(true) => {
                if !check {
                    println!("{label} version already matches ({version})");
                }
            }
            Ok(false) => {
                if check {
                    mismatches.push(format!("{label} version out of sync: {}", path.display()));
                } else {
                    sync_json_at_pointer(path, pointer, &version)?;
                    println!("{label} version updated to {version}");
                }
            }
            Err(e) => return Err(e),
        }
    }

    if check && !mismatches.is_empty() {
        return Err(mismatches);
    }

    if check {
        println!("all versions in sync ({version})");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn hk_cargo_fmt_step_formats_virtual_workspaces() {
        let hk_config = include_str!("../../../hk.pkl");

        assert!(
            hk_config.contains("cargo fmt --check --manifest-path {{workspace_indicator}} --all"),
            "hk cargo-fmt step should use --all when targeting the virtual workspace root"
        );
    }
}

#[cfg(test)]
mod sync_version_tests {
    use std::fs;

    use super::*;

    fn create_temp_workspace(dir: &Path, version: &str) {
        let cargo_toml = format!("[workspace.package]\nversion = \"{version}\"\n");
        fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();

        let plugin_json = "{\n  \"name\": \"tndm\",\n  \"version\": \"0.0.0\"\n}\n";
        fs::write(dir.join("plugin.json"), plugin_json).unwrap();

        let marketplace_json =
            "{\n  \"plugins\": [\n    {\n      \"version\": \"0.0.0\"\n    }\n  ]\n}\n";
        fs::write(dir.join("marketplace.json"), marketplace_json).unwrap();
    }

    #[test]
    fn read_workspace_version_parses_version() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "1.2.3");
        let version = read_workspace_version(&dir.path().join("Cargo.toml")).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn sync_json_at_pointer_updates_top_level_field() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "1.2.3");

        sync_json_at_pointer(&dir.path().join("plugin.json"), "/version", "1.2.3").unwrap();

        let updated: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("plugin.json")).unwrap())
                .unwrap();
        assert_eq!(updated["version"], "1.2.3");
    }

    #[test]
    fn sync_json_at_pointer_updates_nested_field() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "2.0.0");

        sync_json_at_pointer(
            &dir.path().join("marketplace.json"),
            "/plugins/0/version",
            "2.0.0",
        )
        .unwrap();

        let updated: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("marketplace.json")).unwrap())
                .unwrap();
        assert_eq!(updated["plugins"][0]["version"], "2.0.0");
    }

    #[test]
    fn check_json_at_pointer_true_when_matching() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "0.0.0");

        assert!(
            check_json_at_pointer(&dir.path().join("plugin.json"), "/version", "0.0.0").unwrap()
        );
    }

    #[test]
    fn check_json_at_pointer_false_when_mismatched() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "0.0.0");

        assert!(
            !check_json_at_pointer(&dir.path().join("plugin.json"), "/version", "9.9.9").unwrap()
        );
    }
}
