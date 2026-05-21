# Task 2: Enforce closeout evidence and incomplete-task lifecycle guards

## Goal
Move the approved closeout rules from skill prose into enforceable flow-tool behavior.

## Required behavior
- Make `supi_flow_close` reject blank or missing `verification_results`.
- Before closing, inspect the current structured task manifest and refuse to close the ticket while any task is not `done`.
- Keep the successful close path unchanged after validation: create or update `archive.md`, sync the ticket, set `status=done`, and replace flow tags with `flow:done`.
- Reuse shared helpers where it reduces duplication with the new apply/start-close lifecycle checks, but do not broaden the scope into unrelated refactors.

## Test expectations
- Add explicit failing tests for missing `verification_results`.
- Add explicit failing tests for tickets that still contain unchecked tasks.
- Keep existing successful close tests green after the guard logic lands.
