## Design summary
Two-part change:

**Part A (Rust)**: Add `--json` flag to `tndm --version`. When passed, output `{"version":"0.9.0","name":"tndm"}` instead of plain text `tndm 0.9.0`. Plain `--version` behavior unchanged.

**Part B (supi-flow)**: Add version mismatch warning at extension startup. On `session_start` (reason=startup|reload), check `tndm --version --json`. Compare against supi-flow version from package.json. If mismatch, notify via PI. Falls back to text parsing when `--json` is unsupported (older tndm).
