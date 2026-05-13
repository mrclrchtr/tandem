supi_flow_plan unconditionally sets flow:planned and only removes flow:brainstorm, but when re-persisting a plan during the apply phase (when ticket has flow:applying), it doesn't clean up the flow:applying tag — resulting in two flow-state tags. The fix should ensure supi_flow_plan replaces whatever flow-state tag exists with flow:planned, and supi_flow_close should also clean up all flow-state tags consistently.

Relevant files:
- plugins/supi-flow/extensions/tools/flow-tools.ts (executeFlowPlan, executeFlowClose)
- plugins/supi-flow/__tests__/flow-tools.test.ts (tests to update)
