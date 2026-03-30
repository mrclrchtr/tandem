# Version Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep the plugin version (`plugin.json`) and marketplace version (`marketplace.json`) in sync with the Cargo.toml workspace version via a single xtask command.

**Architecture:** Add a `sync-version` subcommand to the existing `xtask` crate. It reads `workspace.package.version` from the root `Cargo.toml` and writes it to both JSON files. A `--check` flag enables dry-run mode for CI. Mise tasks wrap the command for convenience, and CI runs the check.

**Tech Stack:** Rust, `toml` crate, `serde_json` crate (both already workspace deps), mise, GitHub Actions.

**User Verification:** NO — no user feedback or human-in-the-loop validation required.

---

### Task 1: Add dependencies to xtask

**Goal:** Add `toml` and `serde_json` workspace dependencies to the xtask crate.

**Files:**
- Modify: `crates/xtask/Cargo.toml`

**Acceptance Criteria:**
- [ ] `xtask` compiles with `toml` and `serde_json` dependencies
- [ ] `cargo xtask check-arch` still passes (xtask → toml/serde_json is not a forbidden edge since those are external crates, not workspace crates)

**Verify:** `cargo build -p xtask` → exits 0

**Steps:**

- [ ] **Step 1: Add dependencies to xtask/Cargo.toml**

Add `toml` and `serde_json` to the `[dependencies]` section:

```toml
[dependencies]
cargo_metadata.workspace = true
serde_json.workspace = true
toml.workspace = true
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build -p xtask`
Expected: exits 0

- [ ] **Step 3: Commit**

```bash
git add crates/xtask/Cargo.toml
git commit -m "build(xtask): add toml and serde_json dependencies for version sync"
```

---

### Task 2: Implement sync-version with tests

**Goal:** Implement the `sync-version` subcommand with `--check` flag and unit tests.

**Files:**
- Modify: `crates/xtask/src/main.rs`

**Acceptance Criteria:**
- [ ] `cargo xtask sync-version` reads workspace version from Cargo.toml and writes to both JSON files
- [ ] `cargo xtask sync-version --check` exits 0 when in sync, exits 1 when not
- [ ] Unit tests cover: version read, JSON update, check mode pass/fail
- [ ] Existing `check-arch` command still works

**Verify:** `cargo test -p xtask` → all tests pass

**Steps:**

- [ ] **Step 1: Write failing tests for the core sync logic**

Add a test module to `crates/xtask/src/main.rs`. The tests need temp directories with a Cargo.toml and two JSON files:

```rust
#[cfg(test)]
mod sync_version_tests {
    use std::fs;

    use super::*;

    fn create_temp_workspace(dir: &std::path::Path, version: &str) {
        let cargo_toml = format!(
            "[workspace.package]\nversion = \"{version}\"\n"
        );
        fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();

        let plugin_json = format!(
            "{{\n  \"name\": \"tndm\",\n  \"version\": \"0.0.0\"\n}}\n"
        );
        fs::write(dir.join("plugin.json"), plugin_json).unwrap();

        let marketplace_json = format!(
            "{{\n  \"plugins\": [\n    {{\n      \"version\": \"0.0.0\"\n    }}\n  ]\n}}\n"
        );
        fs::write(dir.join("marketplace.json"), marketplace_json).unwrap();
    }

    #[test]
    fn read_workspace_version_parses_version() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "1.2.3");
        let version = read_workspace_version(dir.path().join("Cargo.toml")).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn sync_json_field_updates_matching_version() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "1.2.3");

        sync_json_field(dir.path().join("plugin.json"), "version", "1.2.3").unwrap();

        let updated: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("plugin.json")).unwrap()).unwrap();
        assert_eq!(updated["version"], "1.2.3");
    }

    #[test]
    fn check_json_field_returns_true_when_matching() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "0.0.0");

        assert!(check_json_field(dir.path().join("plugin.json"), "version", "0.0.0").unwrap());
    }

    #[test]
    fn check_json_field_returns_false_when_mismatched() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_workspace(dir.path(), "0.0.0");

        assert!(!check_json_field(dir.path().join("plugin.json"), "version", "9.9.9").unwrap());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p xtask sync_version`
Expected: compile errors (functions not defined yet)

- [ ] **Step 3: Implement the core functions**

Add these functions to `crates/xtask/src/main.rs` (before the `#[cfg(test)]` module):

```rust
use std::path::Path;

fn read_workspace_version(cargo_toml_path: std::path::PathBuf) -> Result<String, Vec<String>> {
    let content = std::fs::read_to_string(&cargo_toml_path)
        .map_err(|e| vec![format!("failed to read {}: {e}", cargo_toml_path.display())])?;
    let doc: toml::Table = toml::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", cargo_toml_path.display())])?;
    let version = doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| vec!["missing workspace.package.version in Cargo.toml".to_owned()])?;
    Ok(version.to_owned())
}

fn sync_json_field(path: std::path::PathBuf, field: &str, version: &str) -> Result<(), Vec<String>> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| vec![format!("failed to read {}: {e}", path.display())])?;
    let mut doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", path.display())])?;
    doc[field] = serde_json::Value::String(version.to_owned());
    let output = serde_json::to_string_pretty(&doc)
        .map_err(|e| vec![format!("failed to serialize {}: {e}", path.display())])?;
    std::fs::write(&path, format!("{output}\n"))
        .map_err(|e| vec![format!("failed to write {}: {e}", path.display())])?;
    Ok(())
}

fn check_json_field(path: std::path::PathBuf, field: &str, version: &str) -> Result<bool, Vec<String>> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| vec![format!("failed to read {}: {e}", path.display())])?;
    let doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", path.display())])?;
    Ok(doc.get(field).and_then(|v| v.as_str()) == Some(version))
}
```

- [ ] **Step 4: Add sync-version CLI dispatch**

Update the `run()` function to handle the new subcommand. The function needs the repo root directory. Update the dispatch:

```rust
fn run() -> Result<(), Vec<String>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check-arch") if args.next().is_none() => check_arch(),
        Some("sync-version") => {
            let check = args.next().as_deref() == Some("--check");
            sync_version(check)
        }
        Some(_) => Err(vec![XTASK_USAGE.to_owned()]),
        None => Err(vec![XTASK_USAGE.to_owned()]),
    }
}
```

Update the usage string:

```rust
const XTASK_USAGE: &str = "usage: cargo xtask <check-arch | sync-version [--check]>";
```

Add the `sync_version` function:

```rust
fn sync_version(check: bool) -> Result<(), Vec<String>> {
    let root = find_repo_root()?;
    let version = read_workspace_version(root.join("Cargo.toml"))?;

    let targets = [
        ("plugin", root.join("plugin/tndm/.claude-plugin/plugin.json")),
        ("marketplace", root.join(".claude-plugin/marketplace.json")),
    ];

    let mut mismatches = Vec::new();

    for (label, path) in &targets {
        match check_json_field(path.clone(), "version", &version) {
            Ok(true) => {
                if !check {
                    println!("{label} version already matches ({version})");
                }
            }
            Ok(false) => {
                if check {
                    mismatches.push(format!("{label} version out of sync: {path}"));
                } else {
                    sync_json_field(path.clone(), "version", &version)?;
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

fn find_repo_root() -> Result<std::path::PathBuf, Vec<String>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| vec!["CARGO_MANIFEST_DIR not set".to_owned()])?;
    let root = std::path::PathBuf::from(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| vec!["cannot determine repo root".to_owned()])?
        .to_owned();
    Ok(root)
}
```

Note: for the marketplace.json, the version field is nested under `plugins[0].version`. The `check_json_field` / `sync_json_field` functions currently operate on top-level fields. We need to handle the nested case. Update `sync_json_field` and `check_json_field` to accept a path to the field (or handle the nesting). Simplest approach: for marketplace.json, sync `plugins[0].version`:

```rust
fn sync_json_at_pointer(path: std::path::PathBuf, pointer: &str, version: &str) -> Result<(), Vec<String>> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| vec![format!("failed to read {}: {e}", path.display())])?;
    let mut doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", path.display())])?;
    doc.pointer_mut(pointer)
        .ok_or_else(|| vec![format!("JSON pointer {pointer} not found in {}", path.display())])?
        .as_string_mut()  // This doesn't exist; use a different approach
        .map(|_| ())
}
```

Actually, let's simplify. Use a JSON pointer for the marketplace file. Replace the separate `check_json_field` / `sync_json_field` with a unified approach that uses JSON pointers:

```rust
fn sync_json_at_pointer(path: &Path, pointer: &str, version: &str) -> Result<(), Vec<String>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| vec![format!("failed to read {}: {e}", path.display())])?;
    let mut doc: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| vec![format!("failed to parse {}: {e}", path.display())])?;
    let target = doc.pointer_mut(pointer).ok_or_else(|| {
        vec![format!("JSON pointer {pointer} not found in {}", path.display())]
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
```

Then update the targets to use JSON pointers:

```rust
let targets = [
    ("plugin", root.join("plugin/tndm/.claude-plugin/plugin.json"), "/version"),
    ("marketplace", root.join(".claude-plugin/marketplace.json"), "/plugins/0/version"),
];
```

Update the loop accordingly:

```rust
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
```

- [ ] **Step 5: Update tests to use JSON pointer functions**

Rewrite the tests to test `sync_json_at_pointer` and `check_json_at_pointer`:

```rust
#[test]
fn sync_json_at_pointer_updates_field() {
    let dir = tempfile::tempdir().unwrap();
    create_temp_workspace(dir.path(), "1.2.3");

    sync_json_at_pointer(&dir.path().join("plugin.json"), "/version", "1.2.3").unwrap();

    let updated: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("plugin.json")).unwrap()).unwrap();
    assert_eq!(updated["version"], "1.2.3");
}

#[test]
fn sync_json_at_pointer_updates_nested_field() {
    let dir = tempfile::tempdir().unwrap();
    create_temp_workspace(dir.path(), "2.0.0");

    sync_json_at_pointer(&dir.path().join("marketplace.json"), "/plugins/0/version", "2.0.0").unwrap();

    let updated: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("marketplace.json")).unwrap()).unwrap();
    assert_eq!(updated["plugins"][0]["version"], "2.0.0");
}

#[test]
fn check_json_at_pointer_true_when_matching() {
    let dir = tempfile::tempdir().unwrap();
    create_temp_workspace(dir.path(), "0.0.0");

    assert!(check_json_at_pointer(&dir.path().join("plugin.json"), "/version", "0.0.0").unwrap());
}

#[test]
fn check_json_at_pointer_false_when_mismatched() {
    let dir = tempfile::tempdir().unwrap();
    create_temp_workspace(dir.path(), "0.0.0");

    assert!(!check_json_at_pointer(&dir.path().join("plugin.json"), "/version", "9.9.9").unwrap());
}

#[test]
fn read_workspace_version_parses_version() {
    let dir = tempfile::tempdir().unwrap();
    create_temp_workspace(dir.path(), "1.2.3");
    let version = read_workspace_version(dir.path().join("Cargo.toml")).unwrap();
    assert_eq!(version, "1.2.3");
}
```

Note: `tempfile` is already a workspace dependency. Add it to `xtask/Cargo.toml` as a dev-dependency:

```toml
[dev-dependencies]
tempfile.workspace = true
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p xtask`
Expected: all tests pass

- [ ] **Step 7: Verify existing check-arch still works**

Run: `cargo xtask check-arch`
Expected: "architecture checks passed"

- [ ] **Step 8: Commit**

```bash
git add crates/xtask/src/main.rs crates/xtask/Cargo.toml
git commit -m "feat(xtask): add sync-version subcommand with --check flag"
```

---

### Task 3: Add mise tasks

**Goal:** Add `sync-version` and `bump` mise tasks, and update the `check` task to include version sync verification.

**Files:**
- Modify: `mise.toml`

**Acceptance Criteria:**
- [ ] `mise run sync-version` runs `cargo xtask sync-version`
- [ ] `mise run bump -- <version>` updates Cargo.toml workspace version and propagates
- [ ] `mise run check` includes version sync check

**Verify:** `mise run sync-version` → updates both JSON files; `mise run check` → passes

**Steps:**

- [ ] **Step 1: Add mise tasks to mise.toml**

Append after the `[tasks.arch]` section:

```toml
[tasks.sync-version]
description = "Sync plugin/marketplace versions from Cargo.toml"
run = "cargo xtask sync-version"

[tasks.bump]
description = "Bump workspace version and sync derived files"
run = """
version="${@}"
if [ -z "$version" ]; then
  echo "usage: mise run bump -- <version>" >&2; exit 1
fi
sed -i '' -e "s/^version = \".*\"/version = \"${version}\"/" Cargo.toml
mise run sync-version
"""

[tasks.check]
description = "Run all checks"
depends = ["fmt", "compile", "arch", "clippy", "test"]
run = "cargo xtask sync-version --check"
```

Note: The `check` task currently uses `depends` only. We need to change it to run the sync-version check after the depends complete. The simplest approach: add a separate task for the check and chain them:

Actually, let's keep it simpler. Just add sync-version check as a separate task and add it to depends:

```toml
[tasks.sync-version-check]
description = "Check plugin/marketplace versions match Cargo.toml"
run = "cargo xtask sync-version --check"

[tasks.check]
description = "Run all checks"
depends = ["fmt", "compile", "arch", "clippy", "test", "sync-version-check"]
```

And the `sync-version` and `bump` tasks remain as shown.

- [ ] **Step 2: Verify mise tasks**

Run: `mise run sync-version`
Expected: updates both JSON files with version from Cargo.toml

Run: `mise run check`
Expected: all checks pass (including sync-version check)

- [ ] **Step 3: Commit**

```bash
git add mise.toml
git commit -m "feat(mise): add sync-version, bump, and version check tasks"
```

---

### Task 4: Add CI step

**Goal:** Add `cargo xtask sync-version --check` to the CI workflow.

**Files:**
- Modify: `.github/workflows/ci.yml`

**Acceptance Criteria:**
- [ ] CI runs version sync check after architecture check

**Verify:** `cargo xtask sync-version --check` → exits 0 when in sync

**Steps:**

- [ ] **Step 1: Add step to ci.yml**

Insert after the "Architecture" step:

```yaml
      - name: Version sync
        run: mise run sync-version-check
```

- [ ] **Step 2: Verify YAML is valid**

Run: `mise run compile` (ensures no syntax issues in the repo)
Expected: exits 0

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add version sync check step"
```

---

### Task 5: Initial sync

**Goal:** Run `sync-version` to align all versions, then verify the full check suite passes.

**Files:**
- Modify: `plugin/tndm/.claude-plugin/plugin.json` (version updated by sync-version)
- Modify: `.claude-plugin/marketplace.json` (version updated by sync-version)

**Acceptance Criteria:**
- [ ] Both JSON files contain the workspace version (`0.1.0`)
- [ ] `mise run check` passes

**Verify:** `mise run check` → exits 0

**Steps:**

- [ ] **Step 1: Run sync-version**

Run: `mise run sync-version`
Expected: prints that both files were updated to `0.1.0`

- [ ] **Step 2: Verify files**

Run: `grep '"version"' plugin/tndm/.claude-plugin/plugin.json .claude-plugin/marketplace.json`
Expected: both show `"0.1.0"` (or equivalent for the marketplace's nested field)

- [ ] **Step 3: Run full check suite**

Run: `mise run check`
Expected: all checks pass

- [ ] **Step 4: Commit**

```bash
git add plugin/tndm/.claude-plugin/plugin.json .claude-plugin/marketplace.json
git commit -m "chore: sync plugin and marketplace versions to 0.1.0"
```
