---
id: T-4148
title: "xlsx export keeps empty-string cells (t=s to an empty string) instead of writing them blank"
milestone: M11
lane: engine
status: done
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
- [x] After load and save, every cell of poi/bug66675.xlsx that held "" still holds "" (test), and ISBLANK of such a
      cell is FALSE after a reload.
- [x] The census cases above have no missing cell left (`rustsheet-eval snapshot` before and after).
- [x] `make check` green; reviewers approve.

## Out of scope
- Style-only blank cells.

## Outcome
Already fixed on main before this task was picked (origin/main ee54bddd5); this PR adds the regression test and closes
the task. The 2026-11-04 census ran on ee79bfa4. Since then, the round-trip harness has treated an empty-string
constant as not blank (`stored_facts`, crates/rustsheet-eval/src/roundtrip/semantic.rs, SPEC-0033: "An empty-string cell
is not a blank cell"), and the engine keeps these cells.
- bug66675.xlsx stores them as `t="str"` with no `<v>` (B9, C9, D9) and as an `inlineStr` with an empty `<t>` (E9). The
  import reads both as the shared string "", and the export writes `t="s"` to "". `rustsheet-eval snapshot` reads the
  original's B9 and E9 as `{"t":"s","v":""}`.
- New test `crates/rustsheet-xlsx/tests/empty_string_cells.rs`: `t="str"` with no value and with an empty `<v>`, and
  `inlineStr` with an empty and a self-closed `<t>`. All four stay text "" after load, after save and after reload, where
  `OR(ISBLANK(...))` is FALSE and COUNTA counts 4.
- `rustsheet-eval roundtrip` on the census cases, 100.00% kept: poi bug66675 (32,038 items, 0 failing), 52348 (0
  failing), 60512.xlsm (0 failing), calipers pv23_056 (0 failing) and corpus-bench 124-46 (0 failing). xssf-enum.xltx
  keeps 100% of its items; its one failure is an unrelated package check, a sheet name ending in `_x000a_`.
- Eval delta: none. The test is the only code change.
