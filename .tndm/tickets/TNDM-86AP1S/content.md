Remove two unused/over-engineered features from plugins/supi-flow/:
1. The supi-flow-slop-detect automation scripts (4 TypeScript files + vocabulary.json, ~1,100 lines) — keep SKILL.md with manual guidance
2. The doc_create and sync actions from supi_tndm_cli tool surface — these are internal implementation details leaked as agent-facing tool actions
