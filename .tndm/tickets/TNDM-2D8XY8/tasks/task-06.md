# Task 6: Update CLI tests: remove clear tests, update add/edit tests for mandatory detail docs

## Goal

Remove tests for deleted functionality, update tests that exercise removed parameters.

## Change

- Remove `task_detail_clear_detaches_link_without_deleting_doc` test function (and its `#[allow]` attribute).
- Remove any other test that exercises `TaskDetailCommand::Clear`.
- Update `task_detail_ensure_*` tests if they rely on `--detail-path` flag.
- Update `handle_task_add`-related tests: any test passing `--detail-path` must be updated or removed.
- Update `handle_task_edit`-related tests: same.
- Add a test verifying that `task add` automatically creates `tasks/task-01.md` and sets `detail_path`.
- Run `cargo test -p tandem-cli` to verify all pass.
