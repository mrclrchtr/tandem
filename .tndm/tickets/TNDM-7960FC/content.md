## Brainstorming Outcome
**Problem**: Users who install supi-flow via npm don't get a clear signal that tndm is a required runtime dependency. Failure mode is a cryptic shell error.

**Recommended approach**: Status quo (tndm on PATH) with two improvements:
1. postinstall check-only script — warns if tndm not found (read-only, no download)
2. Helpful runtime error — when exec fails with ENOENT, surface clear install instructions

**Why**: Zero new attack surface. No binary downloads, no chmod, no network.
