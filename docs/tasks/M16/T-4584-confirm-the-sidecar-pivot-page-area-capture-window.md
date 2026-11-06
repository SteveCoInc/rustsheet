---
id: T-4584
title: "Confirm the sidecar pivot page-area capture window with sentinel cells outside it"
milestone: M16
lane: harness
status: todo
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
- [ ] A filled probe reads every cell in a margin of at least two rows and two columns around the window for at least three page shapes, and finds no page cell outside it. Otherwise snapshot.py's window is widened, with a test.
- [ ] The pivots.md entry's window status becomes VERIFIED or is corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine rendering of page fields (T-4149).
