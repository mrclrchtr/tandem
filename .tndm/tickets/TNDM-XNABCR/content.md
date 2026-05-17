Three verified findings from code review of structured task management:
1. handle_task_add doesn't normalize empty optional strings to None (asymmetry with handle_task_edit)
2. No test for completing an already-done task (idempotency untested)
3. 3 integration tests use show --json instead of task list --json to verify task state
