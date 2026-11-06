---
id: T-4477
title: "FILTER include edges left unprobed by T-3506: a one-cell error include that fits a one-row or one-column array, and a wrongly shaped multi-cell include holding an error"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FILTER gives Excel's result for the include (and array) forms T-3506 left unprobed: a one-cell error include whose
1x1 shape fits a one-row or one-column array, a wrongly shaped multi-cell include holding an error, and a one-cell
error reached through a LET variable, INDEX or IF rather than written as a reference. The rules are probed first,
then implemented in `fn_filter`, and recorded in docs/excel-semantics/arrays.md.

## Context
- arrays.md "A one-cell error reference is a 1x1 array in TOCOL, TOROW and FILTER (T-3497)" (gen-m7 arg-pairs batch,
  excel-graph, 2026-10-31): with Data!G9 `#DIV/0!`, `FILTER(Data!A1:D12,Data!G9,Data!G3)` is `#VALUE!` (gen-m7/
  d7a127bd9334c2b4 A2380), `FILTER(Data!G9,Data!G2)` `#CALC!` (68e7f10771a7118a A10798), a literal error is still the
  error (ee81c689b5fde629 A977). Its "Ours" paragraph ends: "Unprobed, so kept as before: a one-cell error include
  that fits the array's shape (a one-row or one-column array) is its error, and a wrongly shaped multi-cell include
  holding an error still returns the error first."
- T-3506's Outcome: "a wrongly shaped multi-cell include holding an error keeps its old order (unprobed)", pinned only
  by the unit test `filter_wrongly_shaped_range_include_with_an_error_is_the_error`.
- Ours: `Model::fn_filter` and `include_axis` (crates/rustsheet-engine/src/functions/lookup_and_reference/filter.rs)
  check a wrapped one-cell error include's shape first, then every element of any include (an error anywhere is
  returned), then the shape. `wrap_one_cell_error_ref` (functions/spill_functions.rs) wraps only a written
  `Node::ReferenceKind`; T-3765 added `Model::is_one_cell_variable` so VSTACK/HSTACK also wrap a LET variable or
  LAMBDA parameter bound to such a reference, which FILTER does not use. A 1x1 include against a one-column array of
  several rows is read by `include_axis` as a column filter (width 1 = the array's one column), so
  `FILTER(A1:A3,Data!G9)` is `#DIV/0!` here.
- Write one probe case, probes/t4477-filter-include-error-shapes (excel-probe skill, excel-graph, tag M7), G9 `#DIV/0!`, G7 and G8 1,
  G2 0, A1:C3 numbers; each result read through `ROWS(..)&"x"&COLUMNS(..)`, `ERROR.TYPE(..)` and `ARRAYTOTEXT(..)`:
  - a fitting 1x1 error include: `FILTER(A1:C1,G9)`, `FILTER(A1:A3,G9)`, `FILTER(A1:A1,G9)`, `FILTER(A1:C1,G9,"e")`;
  - a wrongly shaped include holding an error: `FILTER(A1:A3,{1;#N/A})`, `FILTER(A1:A3,{#N/A,1})`, `FILTER(A1:A3,G8:G9)`
    and `FILTER(A1:C3,G8:G9,"e")`, beside the right-shaped controls `FILTER(A1:A3,{1;#N/A;1})` and `FILTER(A1:A3,G7:G9)`;
  - an error reached indirectly: `FILTER(A1:C3,LET(x,G9,x))`, `FILTER(A1:C3,INDEX(G1:G9,9))`, `FILTER(A1:C3,IF(TRUE,G9))`,
    `MAP(1,LAMBDA(p,FILTER(A1:C3,G9)))` as a control, and as the array `FILTER(LET(x,G9,x),G2)` and
    `FILTER(INDEX(G1:G9,9),TRUE)`.

## Acceptance criteria
- [x] probes/t4477-filter-include-error-shapes is committed with oracle values (excel-graph) for every formula above.
- [x] Every cell of it passes (`make eval-show CASE=probes/t4477-filter-include-error-shapes`); any new wrapping reuses
      `wrap_one_cell_error_ref` / `Model::is_one_cell_variable` (no second copy), commented with the probe cells.
- [x] gen-m7 (the T-3506 cells above in particular), probes/t3654-filter-result-types and
      probes/one-cell-error-ref-array-functions still pass; the eval gate shows 0 regressions in every suite.
- [x] docs/excel-semantics/arrays.md: the "Unprobed, so kept as before" sentence of the T-3497 entry is replaced by the
      probed rules with their cells; `filter_wrongly_shaped_range_include_with_an_error_is_the_error` is kept, renamed
      or changed to match the evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- TOCOL, TOROW, SORT and the other array functions' one-cell error references (T-3506, T-3667, done).
- GROUPBY and PIVOTBY filters (T-4464, T-3892); FILTER's if_empty argument kinds.
- Errors read through a reference other than a stored cell error (a circular read): no oracle form for it.
- PIVOTBY subtotal work (T-4388, T-4401, T-4414, T-4428), function arrays (T-4449) and eta names (T-4450) are unrelated.

## Outcome
Probed (probes/t4477-filter-include-error-shapes, excel-graph, 49 cells) and implemented.
- Excel checks FILTER's include shape before its elements for every include: a wrongly shaped include holding an error
  is `#VALUE!` (J27:L39), a fitting one-cell error include is its error (J11:L23), a right-shaped include holding an
  error is the error (J43:L47).
- A one-cell error reference reached through LET, INDEX or IF is a 1x1 array, as array and as include (J51:L59,
  J67:L71). FILTER now reads both arguments as references (`Model::filter_arg_array`, through the existing
  `evaluate_picked_reference_arg`), which replaces the written-reference-only wrapping; `wrap_one_cell_error_ref`'s
  `wrapped` flag had no other user and is folded into `one_cell_error_ref_as_array` (TOCOL/TOROW/CHOOSE* unchanged).
- `filter_wrongly_shaped_range_include_with_an_error_is_the_error` became `..._is_value` (its old A1:B3 against a
  2-row include was a column filter under `include_axis`, so it now uses A1:A3); a new unit test covers the indirect
  forms. arrays.md T-3497 entry: the "Unprobed, so kept as before" sentence is replaced by the probed rules.
- Eval delta: probes +1 case, 49/49 cells pass; 0 regressions in every suite (gen-m7, t3654-filter-result-types and
  one-cell-error-ref-array-functions included).
