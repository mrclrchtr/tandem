# Task 5: Refactor validate_registered_task_detail_path into ensure_canonical_task_detail_doc

## Goal

The function `validate_registered_task_detail_path` currently validates an existing path — it errors if the doc doesn't exist or isn't registered. Its role should flip: ensure the canonical doc exists, register it, and return the path. Rename to `ensure_canonical_task_detail_doc`.

## Change

- Rename `validate_registered_task_detail_path` to `ensure_canonical_task_detail_doc`.
- Change its signature: instead of accepting `detail_path: Option<String>` and returning `Result<Option<String>>`, accept `title: &str` (for the template) and return `Result<String>` (the canonical relative path, always present).
- Inline the file-creation, document-registration, and fingerprint-recompute logic from `handle_task_detail_ensure` (minus the JSON output).
- Callers: `handle_task_add`, `handle_task_set` use this helper. `handle_task_edit` no longer needs it.
