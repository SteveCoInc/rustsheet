---
id: T-4479
title: "SORT and SORTBY bad-argument order left unprobed: a text or computed #VALUE! sort_order before a later error, and three or more bad arguments, probe and match"
milestone: M7
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Which of several bad arguments SORT and SORTBY report matches Excel for the forms docs/excel-semantics/arrays.md still
marks UNVERIFIED: a sort_order that is text or a computed `#VALUE!` against a later error argument, and three or more
bad arguments in one call. The cases are evidenced on excel-graph and the engine follows them through one reading
order per function.

## Context
- arrays.md, "SORT argument errors come before validation" ("Ours" paragraph): "A #VALUE! sort_order is still
  returned after by_col's error (multi-cell order of c5d278a8662ef4b2 A39 needs it; a real `"a"+1` order against a
  later error argument is UNVERIFIED for both SORT and SORTBY, which returns it at once)."
- arrays.md, "TOCOL, TOROW and SORT: which bad argument wins (T-3497)": "The order among more than two bad arguments
  is UNVERIFIED beyond these cells."
- The two functions read text orders differently today, and only SORT's reading is evidenced:
  - SORT: `sort_number_list` (crates/rustsheet-engine/src/functions/spill_functions.rs) turns a text or FALSE
    sort_index or sort_order into NaN, which is validated only after by_col is read. So
    `SORT(Data!B10,"abc",0,#NUM!)` is `#NUM!` (gen-m7/91b6b88061b805b1 A11103, VERIFIED). An error value is itself, at
    once.
  - SORTBY: `sortby_of` casts each sort_order with `cast_to_number` as it walks the pairs, so a text order returns
    `#VALUE!` before a later pair's by_array error is seen. That is the opposite of SORT's evidenced rule, and no cell
    covers it (probes/sort-error-args H77-H105 has error orders and an invalid number, 0, but no text order).
- Probe (excel-probe skill, excel-graph, tag M7), with A1:A3 = 3, 1, 2, C1:C3 = "c", "a", "b", each result read
  through `ERROR.TYPE` (as probes/sort-error-args does):
  - SORT, a computed error before a later error: `SORT(A1:A3,1,"a"+1,NA())`, `SORT(A1:A3,"a"+1,1/0)`,
    `SORT(A1:A3,1,"a"+1,{TRUE,FALSE})` (lifted by_col);
  - SORTBY, text and computed orders: `SORTBY(A1:A3,C1:C3,"a",C1:C3,NA())`, `SORTBY(A1:A3,C1:C3,"a"+1,C1:C3,NA())`,
    `SORTBY(A1:A3,C1:C3,"a",NA(),1)` (the error in the next by_array), `SORTBY(A1:A3,C1:C3,"a",C1:C2,1)` (a later
    size mismatch), `SORTBY(A1:A3,C1:C3,TRUE,C1:C3,NA())`;
  - three or more bad arguments, each kind in each position: an error, a blank cell, text, an out-of-range number
    (0 or 5) and an invalid by_col (`"x"`), for example `SORT(A1:A3,"x",0,NA())`, `SORT(A1:A3,5,"x",1/0)`,
    `SORT(A1:A3,0,Z1,NA())` with Z1 blank, `SORTBY(A1:A3,C1:C3,0,C1:C3,"x",C1:C3,NA())`,
    `SORTBY(A1:A3,C1:C2,"x",C1:C3,1/0)`.
- Write the engine's values as predictions before the fill (docs/research/t4479/). The rule is chosen on these cells
  (structural, not a fit), so give it a small hold-out: a second, generated set of three-argument combinations,
  predictions frozen before its fill (decision 0188 R4).

## Acceptance criteria
- [x] probes/t4479-sort-bad-argument-order (excel-graph, canary-verified) covers every form in Context; its
      predictions are committed before the fill, and the hold-out case's predictions before its fill.
- [x] Every cell of both cases passes, or each cell that does not is recorded in arrays.md and owned by a filed task
      named in the Outcome.
- [x] arrays.md: the two UNVERIFIED sentences quoted in Context are replaced by the evidenced rule, with cells cited.
- [x] If SORT and SORTBY turn out to share one reading order, they read it through one helper (directive 3), not two
      copies; unit tests in test_spill_functions.rs pin each rule.
- [x] No regressions in any baseline; the cells of probes/sort-error-args, t2929, t2947 and t4447 still pass;
      `make check` green; reviewers approve.

## Out of scope
- Fractional and near-integer sort_order and sort_index (T-4447, done).
- A lone blank cell as SORT's array (T-4448).
- The three SORTBY forms Excel for the web refuses to load (arrays.md, desktop question, decision 0039).

## Outcome
Probed; the engine already matches every cell, so no engine code changed.
- probes/t4479-sort-bad-argument-order (93 cells: every form in Context, then a grid of a text, out-of-range, blank or
  error value per argument, each argument with its own error) and the hold-out probes/t4479-sort-bad-argument-holdout
  (70 cells, a `random.Random(4479)` sample of other combinations with other values). Both were filled on excel-graph
  (`rustsheet-oracle fill`, which canary-verifies every workbook; 0 failures). `docs/research/t4479/gen_probe.py` writes them.
- Predictions: the engine's values for both cases were committed (`docs/research/t4479/predictions-*.tsv`, commit
  "probe cases and the engine's predictions, before the fill") before either fill, and the hold-out was filled only
  after the main case had been scored. Main 93/93 and hold-out 70/70 match the predictions. The rule is structural
  (which argument stops the read), not a numeric fit, so no `FITTED` tag applies.
- Rule found: SORT and SORTBY do not share a reading order, so there is no shared helper to extract. SORT stops at an
  error or a blank sort_index/sort_order, and text fails only in validation. SORTBY stops at an error or a non-numeric
  text order, and a blank, FALSE or out-of-range order fails only in validation. arrays.md: the two UNVERIFIED
  sentences are replaced, and the new section "SORT and SORTBY: which of several bad arguments wins (T-4479)" cites
  the cells.
- Unit test `sort_and_sortby_read_a_text_sort_order_apart` (test_spill_functions.rs) pins both rules.
- Eval delta: +163 passing probe cells in two new cases (eval/baseline/probes.txt records them), 0 regressions.
