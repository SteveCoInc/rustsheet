---
id: T-4584
title: "Confirm the sidecar pivot page-area capture window with sentinel cells outside it"
milestone: M16
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Evidence beyond consistency that the sidecar page-area capture window (`snapshot.py` `pivot_area`: `rowPageCount` rows above the location minus one blank row, `3 * colPageCount - 1` columns wide) holds every page cell Excel renders. Or, if it does not, the window is widened.

## Context
probes/t4137-pivot-captions K4 and probes/t4138-pivot-geometry G8 are consistent with the window. But a capture cannot see past its own window: row 1 of K4 and rows 1–4 of G8 were never read (Excel-fidelity reviews of T-4137 and T-4138). Read cells just outside the window, for example with `check` formulas such as `=G8!A4` or `=ISBLANK(...)` on the spec case, or capture a wider area once. Cover pageWrap 1, 2 and 3 and pageOverThenDown on and off. docs/excel-semantics/pivots.md "Page fields render above the location".

## Acceptance criteria
- [x] A filled probe reads every cell in a margin of at least two rows and two columns around the window for at least three page shapes, and finds no page cell outside it. Otherwise snapshot.py's window is widened, with a test.
- [x] The pivots.md entry's window status becomes VERIFIED or is corrected.
- [x] `make check` green
- [x] reviewers approve.

## Out of scope
- Engine rendering of page fields (T-4149).

## Outcome
probes/t4584-pivot-page-window (excel-graph): nine pivots at D12 with 1 to 6 page fields, `pageWrap` 1, 2 and 3 and none, over-then-down and down-then-over. Formulas on other sheets read every cell of rows 1–15 from column A to at least three columns past the window, with `ISBLANK` and `&""`. Excel's saved `rowPageCount` by `colPageCount` ranged from 1 by 1 to 6 by 1, 2 by 3 and 1 by 4. No non-blank cell lies outside the `snapshot.py` window, so the window is unchanged. pivots.md "Page fields render above the location" now marks the window VERIFIED. The same case also shows the down-then-over grids (PivE, PivG) and a no-wrap row (PivH), which T-4650's page-grid bullet can cite. The engine passes every cell (3280 of 3280).
