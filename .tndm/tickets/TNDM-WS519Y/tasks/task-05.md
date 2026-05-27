# Task 5: Final verification: compile, lint, test, arch, and smoke

Run the full CI-style verification suite:

```sh
cargo build                          # Compile all crates
mise run fmt                         # Check formatting
mise run clippy                      # Lint with warnings as errors
mise run test                        # Full test suite
mise run arch                        # Workspace architecture invariants
```

Then smoke-test the binary:

```sh
./tndm-dev ticket list
./tndm-dev ticket create "smoke test refactor"
./tndm-dev ticket show <ID>          # from output above
./tndm-dev ticket update <ID> --status done
./tndm-dev fmt --check               # canonical format check
```

All commands must succeed. No behavioral changes — output format identical to pre-refactor.

**Verification**: All checks pass, binary produces correct output.
