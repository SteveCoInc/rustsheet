---
id: T-4148
title: "xlsx export keeps empty-string cells (t=s to an empty string) instead of writing them blank"
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A cell holding the empty string (a `t="s"` cell whose shared string is empty, or an inline empty string) stays a
text cell holding "" after load and save, not a blank cell.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
poi/bug66675.xlsx loses 3,604 such cells (`WGT-D01 By Production Day` B9 and others). poi 52348, 60512.xlsm and
xssf-enum.xltx, calipers pv23_056 and corpus-bench 124-46 lose a few. The difference is visible in Excel: ISBLANK is FALSE
and COUNTA counts the cell. Find whether import or export drops it. Check docs/excel-semantics/ for the empty-string
rules (M1 literals).

## Acceptance criteria
- [ ] After load and save, every cell of poi/bug66675.xlsx that held "" still holds "" (test), and ISBLANK of such a
      cell is FALSE after a reload.
- [ ] The census cases above have no missing cell left (`rustsheet-eval snapshot` before and after).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Style-only blank cells.
