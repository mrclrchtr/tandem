# Archive

- `cargo test -p tandem-cli ticket_doc_create_rejects_existing_registered_path -v`
- `cargo test -p tandem-cli -v`
- `(cd plugins/supi-flow && pnpm exec vitest run __tests__/tndm-cli-tool.test.ts)`

Results:
- duplicate document-path creation now fails instead of overwriting registered files like `content.md`
- task add/edit/set now reject dangling `detail_path` values unless they reference the canonical registered task detail document lifecycle
- updated CLI regressions and plugin wrapper tests passed
