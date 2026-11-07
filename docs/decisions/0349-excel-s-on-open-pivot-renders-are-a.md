# 0349 — Excel's on-open pivot renders are a vendored gated suite of corpus copies

- **Status:** accepted (2026-11-07, T-4972).
- **Context:**
  - Check (c) of the pivot corpus (eval-harness.md "Pivot freshness", T-4906) refreshes each
    `refreshOnLoad` pivot. Excel re-renders such a pivot when it opens the file (pivots.md "Excel for the web
    re-renders a refreshOnLoad pivot when it opens the file"), so the saved cells come from before that refresh.
    The check compared them only when the pivot was fresh and its source still held the cache's records, and it
    scored nothing.
  - What Excel shows on open is oracle evidence: `rustsheet-oracle fill` captures every pivot area of the
    recalculated workbook into the sidecar's `pivots` (T-4123). A sidecar has to be committed.
  - The corpus is fetched. Its two suites are ungated (`gate = false`), so they are not in `make check`.
    `oracle_dir` (corpus-bench) could hold the sidecars. But the sidecar's values would then replace Excel's
    saved formula values for the whole case, and the cells would still ratchet in no gate.
  - On 2026-11-07 the corpus's 14 `refreshOnLoad` caches are in four pivot-corpus-closedxml workbooks (MIT).
    pivot-corpus-lo (MPL-2.0) has none.
- **Decision:**
  - Byte-for-byte copies of the corpus workbooks with a `refreshOnLoad` cache may be committed under
    `eval/suites/pivot-on-open/cases/`, with Excel's excel-graph sidecars beside them. Only those workbooks are
    copied. `suite.toml` records each file's licence, upstream repository, pinned revision and upstream path, and
    `LICENSE-ClosedXML` carries the MIT notice.
  - The suite `pivot-on-open` is vendored and gated (`gate = true`, `baseline = "cells"`). When a case's sidecar is
    Excel evidence and holds a pivot's area, check (c) compares the engine's refresh of that pivot with the area,
    whatever the pivot's freshness, and scores the cells. A refresh that fails scores the area against nothing,
    as check (b) does for a pivot the engine does not lay out. Check (b)'s cells for the same pivot are reported
    and not scored.
  - `rustsheet-oracle fill` fills an xlsx case that has pivots but no formula cell (`rustsheet-eval cases` lists
    `pivots`). The suite trusts its cached values, so a refill passes `--include-cached`.
- **Consequences:**
  - This adds a gate and weakens none. `make eval-baseline` generates the baseline
    `eval/baseline/pivot-on-open.txt`, and it may only shrink. The cells the engine does not refresh today are
    known failures with owners (decision 0245): T-4506 owns the 7 pivots over a Table or a name, and T-4677 owns
    the pivot that meets its own merged cells.
  - The copies keep the ClosedXML authors' names that their docProps hold, as published upstream, so that each
    file stays byte for byte the pinned upstream file. They hold no other personal data.
  - When the corpus pins move, or a new corpus file gains a `refreshOnLoad` cache, its copy and sidecar are added
    here by the same rule.
