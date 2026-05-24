# Task 8: Add CI frozen-lockfile check for pnpm lockfile drift

Add a step to the existing CI workflow (`.github/workflows/ci.yml`) that verifies the pnpm lockfile is in sync:

```yaml
- name: Check pnpm lockfile (supi-flow)
  run: cd plugins/supi-flow && pnpm install --frozen-lockfile
```

Check whether `pnpm` and Node.js are already available in the CI environment. If not, add a `setup-node` or `setup-pnpm` step before this.

If the existing CI is purely Rust and adding Node/pnpm is disproportionate, add a comment noting this limitation and skip this step (document the decision).

**Verification**: Push a branch with the change; verify CI passes. Or test locally: `cd plugins/supi-flow && pnpm install --frozen-lockfile` exits 0.
