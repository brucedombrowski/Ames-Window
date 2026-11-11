# Released Software Version Descriptions

One file per release, `SVD-vX.Y.Z.md`, written by the `Release` workflow
from the tagged commit and committed to `main` in the commit after the tag.
Each describes the tagged commit, not the commit it lives in, so it never
drifts. `scripts/svd.sh --check` regenerates every file here from its tag
and fails CI if any differ. See DM-2026-003 and issue #10.
