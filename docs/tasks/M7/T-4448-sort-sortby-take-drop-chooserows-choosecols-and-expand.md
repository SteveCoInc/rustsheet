---
id: T-4448
title: "SORT, SORTBY, TAKE, DROP, CHOOSEROWS, CHOOSECOLS and EXPAND of a lone blank cell: probe whether it stays blank as in TOROW and UNIQUE, and match (ours reads 0)"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
SORT, SORTBY, TAKE, DROP, CHOOSEROWS, CHOOSECOLS and EXPAND treat a one-cell reference to a blank cell (and a blank
cell reached through IF or INDEX) as Excel does, so `SORT(Z1)&"x"`, `TYPE(CHOOSEROWS(Z1,1))` and
`TOCOL(EXPAND(Z1,2,1,"p"),1)` give Excel's values. The rule is probed first, then implemented once for every reader,
and recorded in docs/excel-semantics/arrays.md.

## Context
- docs/excel-semantics/arrays.md "TOROW and UNIQUE of a lone blank cell" (T-2409; gen-m7/9bb61f6ae34c7e20 Calc!A3029,
  gen-m7/f7e94a817e5385ac A2942, gen-m9/8dd2312c96effb1b A399, excel-graph): a one-cell reference to a blank cell stays
  blank in TOROW, TOCOL and UNIQUE, as a blank cell of a range would (`TOROW(Z1,3)` is `#CALC!`, `UNIQUE(Z1)&..` joins
  as ""). The entry ends: "SORT and TAKE still read it as 0 and are unprobed (T-2409)".
- Ours: `Model::result_to_array` (crates/rustsheet-engine/src/functions/spill_functions.rs) turns `CalcResult::EmptyCell`
  into `ArrayNode::Number(0.0)`; only `result_to_array_keep_blank` (and its `eval_to_array_keep_blank` /
  `eval_array_arg_keep_blank` wrappers) keeps `ArrayNode::Empty`, used by TOCOL/TOROW (tocol_torow.rs), UNIQUE
  (spill_functions.rs), VSTACK/HSTACK (hstack_vstack.rs) and FILTER's array (filter.rs). Still on the 0 path: SORT and
  SORTBY's array and by_arrays (spill_functions.rs `fn_sort`, `fn_sortby`), CHOOSEROWS and CHOOSECOLS
  (choosecols_chooserows.rs), EXPAND (expand.rs), and the value path of TAKE and DROP (drop_take.rs `result_to_array`
  after `reject_empty_value`).
- Since every other reshaper probed keeps the blank, the likely rule is the same here, but SORTBY with a blank by_array
  element and EXPAND's padding beside a blank may differ; probe before changing anything.
- Write one probe case, probes/t4448-lone-blank-reshapers (excel-probe skill, excel-graph, tag M7), Z1 and Z2 blank,
  each result read three ways so blank and 0 differ: `X&"x"`, `TYPE(X)` (or `ISBLANK(INDEX(X,1,1))`) and
  `TOCOL(X,1)` (drops blanks: `#CALC!` when the blank is the only element):
  - `SORT(Z1)`, `SORT(Z1,1,-1)`, `SORTBY(Z1,Z1)`, `SORTBY(Z1,5)`, `SORTBY({1;2},VSTACK(Z1,1))`;
  - `TAKE(Z1,1)`, `TAKE(Z1,-1)`, `DROP(Z1,0)`, `TAKE(IF(TRUE,Z1),1)`, `DROP(INDEX(Z1:Z2,1),0)`;
  - `CHOOSEROWS(Z1,1)`, `CHOOSECOLS(Z1,1)`, `CHOOSEROWS(Z1,-1)`;
  - `EXPAND(Z1,2)` and `EXPAND(Z1,2,1,"p")` (is the original cell blank and the pad "p");
  - the controls: the same calls over `Z1:Z2` and over `Z1:Z1`, and `TOROW(Z1)` (already evidenced).
  Also ask the WRAPCOLS forms arrays.md "WRAPCOLS of a single value is not padded; wrap_count below 1 is #NUM!; an empty single cell is #VALUE!" leaves unprobed: `WRAPCOLS(IF(TRUE,Z1),2)`,
  `WRAPCOLS(INDEX(Z1:Z2,1),2)` and an omitted vector `WRAPCOLS(,2)` (ours: `#VALUE!` for all three, as for `WRAPCOLS(Z1,2)`).

## Acceptance criteria
- [ ] probes/t4448-lone-blank-reshapers is committed with oracle values (excel-graph) for every formula above.
- [ ] Every cell of it passes (`make eval-show CASE=probes/t4448-lone-blank-reshapers`), each reader taking the blank
      through the existing keep-blank helpers (`result_to_array_keep_blank` and its wrappers), not a new copy; a reader
      whose Excel result is 0 stays on `result_to_array` with a comment citing its probe cell.
- [ ] The eval gate shows 0 regressions in every suite (gen-m7 and probes in particular).
- [ ] docs/excel-semantics/arrays.md: "TOROW and UNIQUE of a lone blank cell" covers the probed functions with their
      cells, and the "SORT and TAKE still read it as 0 and are unprobed" sentence and the WRAPCOLS "Unprobed" sentence are
      replaced by the evidence.
- [ ] A unit test pins `SORT(Z1)&"x"` and `CHOOSEROWS(Z1,1)&"x"` (or their probed values) next to the existing T-2409 tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The empty value (`CELL("contents",<blank>)`, `IFERROR(<blank>,5)`) in these functions: T-3053 (done) and T-3087 (probe).
- Empty text and error arguments of TAKE, DROP, EXPAND and WRAPCOLS: T-2873.
- Blank elements inside a multi-cell range (already kept blank by `evaluate_range`).
