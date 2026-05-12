#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    collections::{HashMap, HashSet},
    env,
    process::ExitCode,
};

use cargo_metadata::MetadataCommand;

const XTASK_USAGE: &str = "usage: cargo xtask <check-arch>";
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
