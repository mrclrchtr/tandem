# Task 3: Migrate ticket_list_tests.rs to TestRepo

## Goal

Migrate `crates/tandem-cli/tests/ticket_list_tests.rs` (396 lines) to use `TestRepo`.

## Files

**`crates/tandem-cli/tests/ticket_list_tests.rs`** — mechanical migration.

### Tests

1. `ticket_list_prints_sorted_rows` — creates two tickets, lists, checks ordering
2. `ticket_list_hides_done_tickets_by_default` — creates + updates + lists
3. `ticket_list_json_hides_done_tickets_by_default` — JSON list with --all toggle
4. `ticket_list_json_outputs_schema_versioned_array` — JSON list envelope
5. `ticket_list_sorts_by_priority_then_id` — creates 4 tickets with priorities
6. `ticket_list_json_empty_produces_empty_array` — list with no tickets
7. `ticket_list_filters_by_definition_tags_in_plain_text` — tag filter
8. `ticket_list_filters_by_definition_tags_in_json` — tag filter with JSON

### Pattern replacements

- Setup → `let repo = TestRepo::new();`
- Ticket creation → `repo.create_ticket(Some("ID"), "Title");`
- Ticket update → `repo.run_assert(&["ticket", "update", "ID", "--status", "done"]);`
- `repo_root.path()` → `repo.path()`
- Output + JSON parsing → use `repo.run_json()` where applicable
- For tests that check human output line-by-line (e.g. `ticket_list_prints_sorted_rows`), use `repo.run_assert()`

### Verification

```bash
cargo test -p tandem-cli ticket_list -- --nocapture
```
