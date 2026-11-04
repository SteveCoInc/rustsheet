---
id: T-3893
title: "PIVOTBY: row sort by an error-holding values column, and a blank relative_to's header text"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
PIVOTBY matches Excel on two cells of the batch: rows sorted by a values column that holds errors, and the header text of a call whose `relative_to` is a blank cell.

## Context
- Found by T-3841 (gen-m7 arg-pairs batch for PIVOTBY, excel-graph oracle, 2026-11-03). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
  PIVOTBY's arguments: row_fields, col_fields, values, function, field_headers, row_total_depth, row_sort_order,
  col_total_depth, col_sort_order, filter_array, relative_to.
- `=_xlfn.PIVOTBY(Data!E1:E12,Data!B1:E12,Data!B1:B12,LAMBDA(acc,acc),0,-1,-2,TRUE,1,(Data!B1:B12<0.5),2)`
  (febfd6901374d79c A11225): row_sort_order -2 sorts the rows by the first values column descending, and that column holds
  `#VALUE!` aggregates (`LAMBDA(acc,acc)` returns an array). Excel's row keys come out 1E+15, 2.5, 42 (A11230:A11232),
  ours 2.5, 42, 1E+15, and the `#VALUE!` cells move with them. Find where Excel sorts an error (probe a small grid).
- `=_xlfn.PIVOTBY(Data!E1:E12,Data!A1:C12,Data!B1:B12,LAMBDA(x," x "),3,1,-2,0,-1,(Data!B1:B12<>2),Data!H1)`
  (ceb2c91035864a71 B8785): Excel's column header joins the col_fields' values as `?b, 3.14159`, ours `, ?b, 3.14159` (a
  leading empty part: a blank key, or the blank `relative_to`?). Probe both.
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs` (`fn_pivotby`), sharing `Model::filter_rows`
  and the GROUPBY helpers in `groupby.rs`.
- Owned cells (2 calls, 12 failing cells; each span runs from the call's anchor over
  the failing cells of its spill or lift):
- gen-m7/ceb2c91035864a71.case.json Calc!A8785:B8785
- gen-m7/febfd6901374d79c.case.json Calc!A11225:F11232

## Acceptance criteria
- [x] The owned cells match Excel; a unit test covers each rule.
- [x] docs/excel-semantics/arrays.md records the PIVOTBY rule (its `Ours:` line) when it is implemented.
- [x] Visible parity: every owned cell has Excel's type and error.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY's other batch rules (T-3888, T-3889, T-3890, T-3892); GROUPBY.

## Progress (2026-11-03, header rule PR)
- Header: done. A blank col_fields header cell is left out of the joined column name (`joined_names`). This fixes
  gen-m7/ceb2c91035864a71.case.json Calc!A8785:B8785; arrays.md has the rule.
- Sort: still open. probes/t3893-pivotby-error-sort-header (excel-graph) shows:
  - GROUPBY and PIVOTBY value sorts already match Excel on error aggregates. Errors are greatest, and a descending sort
    reverses the whole order (A1:A55).
  - The owned call's rows tie: every Total aggregate is `#VALUE!`. Excel's ascending order 0, 42, 2.5, 1E+15 is the exact
    reverse of its descending one, and it is not key order. Find what breaks that tie: probe the same call with other
    filters and LAMBDAs that return a scalar for some rows. Then fix gen-m7/febfd6901374d79c.case.json
    Calc!A11225:F11232 and probes/t3893-pivotby-error-sort-header.case.json Calc!A61:G69, Calc!A71:G79.

## Outcome
- Header rule landed in #2980. Sort rule (this PR): the tie was never a tie. Excel sorts an array aggregate
  (a LAMBDA's array result, shown as `#VALUE!`) by the array, not by `#VALUE!`: after numbers, text and TRUE, before
  errors, two arrays element by element (blank element greatest, a prefix first). `LAMBDA(acc,acc)` therefore sorts the
  rows by their first records' values. New probe probes/t3893-pivotby-error-tie-order (excel-graph, 2026-11-03, 66
  calls) pins it for GROUPBY and PIVOTBY rows and columns; error aggregates keep their old rules.
- Code: `group_order.rs` `aggregate_cmp` (value sorts only; key sorts unchanged), `NestedArray::elements`. Unit test
  `array_aggregates_sort_by_their_elements_between_true_and_errors`. arrays.md updated.
- Eval delta: gen-m7 −11 known-failure cells (febfd6901374d79c A11225:F11232 passes), probes −22
  (t3893-pivotby-error-sort-header A61:G79); the new probe passes 710/710. No regressions.
