## Brainstorming Outcome
**Problem**: Bare IDs like `AC5MQH` are currently treated as literal ticket IDs in CLI lookup/update flows, while canonical stored IDs include the repo-configured prefix (default `TNDM-`). This makes `ticket show AC5MQH` fail when the actual ticket is `TNDM-AC5MQH`.

**Recommended approach**: Add a CLI-local helper that loads the repo config prefix and normalizes bare ticket ID inputs before parsing. If the input already starts with the configured prefix plus `-`, leave it unchanged; otherwise prepend the configured prefix. Use this helper for direct ticket-target commands (`show`, `update`, `sync`, `doc create`) and for `--depends-on` values in `ticket create` / `ticket update`.

**Why**: This keeps stored IDs canonical, respects custom repo prefixes, fixes the user-facing shorthand behavior in the CLI, and avoids pushing repo-config-dependent normalization into `tandem-core`.

**Constraints / non-goals**: Do not hardcode `TNDM`; use the configured prefix. Do not migrate stored ticket IDs. Do not change already-prefixed inputs. Leave explicit `ticket create --id ...` behavior unchanged.

**Open questions**: none.

**Approved by user**: yes.
