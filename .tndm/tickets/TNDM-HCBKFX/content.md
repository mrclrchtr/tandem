## Overview

Apply the two concrete review fixes without broadening scope.

1. In `tndm ticket doc create`, reject a requested `--path` when that ticket-relative path is already registered by any existing document, so the command cannot overwrite `content.md` or alias multiple document names to the same file.
2. In task authoring flows, stop accepting arbitrary `detail_path` persistence. A task detail link must only be created through the canonical `ticket task detail ensure` lifecycle, which also creates/registers the document. Direct task add/edit/set should reject raw `detail_path` input when it does not map to a registered document lifecycle.
3. Add regression coverage in the Rust CLI tests for both behaviors.

## Verification

Run targeted tandem CLI tests that exercise document creation and task detail lifecycle behavior.
