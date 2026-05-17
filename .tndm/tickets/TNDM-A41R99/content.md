Address 4 findings from code review:
1. Empty task title validation in handle_task_add
2. executeFlowPlan silently clears tasks on empty/malformed plan_content
3. Two-call flow-state tag transitions (combine into single calls)
4. handle_task_edit cannot clear optional task fields
