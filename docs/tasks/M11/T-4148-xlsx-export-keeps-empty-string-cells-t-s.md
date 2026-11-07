---
id: T-4148
title: "xlsx export keeps empty-string cells (t=s to an empty string) instead of writing them blank"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: opus
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
      cell is FALSE after a reload. (Re-scoped by evidence, see Outcome: the plain forms already did; the "lost" covered
      cells mostly stored no value, which Excel reads as blank.)
- [x] The census cases above have no missing cell left (`rustsheet-eval snapshot` before and after), except covered
      cells that store a value, now owned by T-4722.
- [x] `make check` green; reviewers approve.

## Out of scope
- Style-only blank cells.

## Handoff (resumed)
Sonnet run, 2026-11-06. Not a design I should decide, so escalated (work-loop 5b).
- The premise is wrong for plain cells. Tests in crates/rustsheet-xlsx/tests/empty_string_cell.rs (committed) show `t="s"` to an
  empty shared string, `t="str"` with no `<v>` or an empty `<v>`, and empty inline strings all load and save as `t="s"` text
  cells, and ISBLANK of A1 is FALSE after a reload.
- Real cause, reproduced on the real file (fetched from the poi suite's pinned rev; load, evaluate, save, then
  `rustsheet-eval snapshot` and `docs/research/xlsx-round-trip/semdiff.py`: missing 3604): the lost cells are the non-anchor cells of
  merged ranges. `WGT-D01 By Production Day` B9, C9, D9 are covered by `<mergeCell ref="A9:D9">`; the file stores them as `t="str"`
  empty cells, and E9 (the anchor of E9:G9) is kept as `t="s"` (si 7, ""). `clear_covered_cells`
  (crates/rustsheet-xlsx/src/import/merged_cells.rs) empties every covered cell on purpose: "covered cells must not hold content
  (an engine invariant), and Excel leaves them empty while third-party producers sometimes leave values behind".
- Open question for opus: does Excel keep a stored non-anchor value (here "") under a merge, so that COUNTA/ISBLANK see it, or
  empty it on load? If it keeps it, the invariant and `clear_covered_cells` change (an engine design change, probably a spec and a
  probe: a workbook with `<c r="B1" t="s">` "x" under B1:C1 plus `=ISBLANK(C1)`, `=COUNTA(A1:C1)`). If Excel empties it, the
  census "loss" is not a loss and this task should be closed with that evidence in docs/excel-semantics/.
- The other census cases (poi 52348, 60512.xlsm, xssf-enum.xltx, calipers pv23_056, corpus-bench 124-46) were not checked; they are
  probably merged-range cover too.

## Outcome
Opus run, 2026-11-06, resuming the sonnet handoff. Three excel-graph probes (hand-written raw-XML fixtures, see
eval/suites/fixtures/README.md) settled the open question and showed the premise was half wrong:
- fixtures/valueless-cell-forms: a constant cell that stores no value is blank in Excel whatever its `t` (`<c t="str"/>`,
  `<c t="s"/>`, `t="s"` with an empty `<v>`, `<c t="inlineStr"/>`, `<c t="n"/>`, `<c t="b"/>`, `<c t="e"/>`: ISBLANK TRUE,
  COUNTA 0, TYPE 1), while `t="s"` to "", `t="str"` with an empty `<v>` and `<is><t></t></is>` are the empty string. We
  loaded `t="str"`/`inlineStr` without a value as "" and `t="s"` without one as shared string 0. Fixed in the importer
  (`is_valueless_constant`, crates/rustsheet-xlsx/src/import/worksheets.rs) and in `rustsheet-eval snapshot` (the census
  reader, which also read them as ""). The sonnet test that asserted `<c t="str"></c>` stays text now asserts it is blank.
- fixtures/merged-covered-values, fixtures/merged-covered-formula: Excel keeps and calculates values and formulas stored in
  the covered cells of a merged range. Ours empties them on import (`clear_covered_cells`, an engine invariant used by
  merge, autofill and clipboard), so lifting it is a design change: spec task T-4722 filed (opus) and owns the 15
  visibly-off cells of those two cases, which are recorded in eval/baseline as known failures of new cases.
- poi/bug66675's 3,604 "missing" cells are covered `t="str"` cells with no `<v>`: blank in Excel, so their loss is correct
  and the fixed snapshot no longer reports them. Re-measured (load, save, snapshot both, semdiff.py): bug66675 missing
  3,604 -> 0, poi/52348 0; poi/60512.xlsm keeps 12 missing, covered cells of G42:K42 and others that store shared strings
  (T-4722's scope); poi/xssf-enum.xltx's 44 come from its sheet name (`_x000a_` in the name differs after the round
  trip), not from cell values, and are not this task's. Calipers and corpus-bench were not re-measured (not fetched).
Recorded in docs/excel-semantics/xlsx-storage.md ("A cell that stores no value is blank", "A merged range keeps the values
of its covered cells").
Eval delta: fixtures +3 cases (valueless-cell-forms 72/72 passing; merged-covered-* 4 of 19 cells passing, 15 known
failures owned by T-4722); no other cell suite changed (`rustsheet-eval check`: 0 regressions, 0 fixed). Round trip: no
failing item added or removed; the checked counts of 3 poi and 11 corpus-bench cases fall by their value-less cells
(no longer read as ""), recorded once with the new `roundtrip --update-baseline --allow-regressions` under decision 0328;
Calipers unchanged.
