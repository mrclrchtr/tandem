# Task 8: Final verification: full test suite, lints, and manual smoke test

## Goal

Run the full validation suite to confirm all refactorings are correct and no regressions.

## Checks

1. **Full test suite:**
   ```sh
   mise run test
   ```
   All tests must pass — workspace-wide.

2. **Lint and architecture:**
   ```sh
   mise run check
   ```
   `fmt`, `compile`, `arch`, `clippy` must all pass. No new warnings.

3. **Canonical format:**
   ```sh
   ./tndm-dev fmt --check
   ```
   Must report "all .tndm files are properly formatted" (or equivalent).

4. **Manual smoke test:**
   ```sh
   ./tndm-dev ticket create "Smoke test" --priority p1 --tags test,refactor --effort s
   # Verify ID printed, files created

   ./tndm-dev ticket update <ID> --status in_progress
   # Verify no error, ID printed

   ./tndm-dev ticket show <ID>
   # Verify priority=p1, tags=[refactor,test], effort=s, status=in_progress

   ./tndm-dev ticket update <ID> --add-tags smoke --remove-tags refactor
   # Verify tags update correctly

   ./tndm-dev ticket task add <ID> --title "Verify refactoring"
   # Verify task created

   ./tndm-dev ticket task complete <ID> 1
   # Verify task marked done
   ```

5. **No leftover imports or dead code:**
   ```sh
   cargo check -p tandem-cli 2>&1 | grep -i "unused\|dead_code\|never_read"
   ```
   Should produce no output.

## Success criteria

- All `mise run check` and `mise run test` pass.
- Manual smoke test covers create, update (with add/remove tags), task add, task complete.
- No dead code warnings.
