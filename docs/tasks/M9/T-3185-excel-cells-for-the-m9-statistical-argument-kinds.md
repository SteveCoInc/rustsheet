---
id: T-3185
title: "Excel cells for the M9 statistical argument kinds no suite covers (573 parameter/kind pairs in 129 functions, mostly empty text and errors)"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every parameter of the 129 M9 statistical functions below has an Excel-backed cell of each argument kind that
report.json `argument_coverage` expects (decision 0094: coverage "across the argument modes its catalog signature
lists"). Every new cell that differs from Excel gets an owner.

## Context
Gap triage 2026-10-30 (`make eval` on main c269641be; decision 0031 evidence gaps). The M2 to M7 argument-kind gaps
have probe tasks (T-2861 to T-2878). M9 has none. Of M9's 1,392 missing parameter/kind pairs, the statistical
category has 662 in 133 functions. Leaving out the FORECAST.ETS family, which is not implemented and waits for desktop
evidence (T-2425, T-1553), leaves 573 pairs in 129 functions. By kind (`classify_value` in
crates/rustsheet-eval/src/argument_coverage.rs): empty_text 285, error 211, numeric_text 35, boolean 12, zero 9,
range 7, text 6, fraction 4, negative 3, blank 1. The full list is report.json `argument_coverage` filtered to catalog
`category: statistical`, `milestone: M9`, `covered < expected`.

Shapes that need care:
- Distribution scalars, for example NORM.DIST x, mean, standard_dev and cumulative; BETA.DIST, BINOM.DIST, GAMMA.DIST,
  T.DIST and the legacy names. Most lack only `""` and an error.
- Array and aggregate parameters, which take a range or array whole: FREQUENCY data_array and bins_array; LARGE and
  SMALL array; PROB x_range and prob_range; T.TEST array1 and array2; CHISQ.TEST and CHITEST actual_range and
  expected_range (numeric text too); LINEST, LOGEST, GROWTH and TREND known_y's, known_x's and new_x's. Excel's rule for a
  `""` or an error inside an array argument is per function (docs/excel-semantics/statistics.md, arrays.md). So each
  needs the kind both as a direct argument and as an array element.
- Criteria parameters of AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS: a boolean, a fraction, numeric text, zero, a negative
  number and (AVERAGEIFS) an error as criteria. Any fix goes to the functions-criteria lane.

The design question is the same as T-3184's: T-2866's `""` and error draws seldom survive `domain_value` and the edge
pools, so new `--focus` batches alone will not reach these kinds. Choose a coverage-directed generator mode,
hand-written excel-probe cases in the T-2861 style (probes/m2-argument-kind-gaps*), or a mix. Record the choice in
docs/architecture/synthetic-generator.md, and share it with T-3184 if that task has landed. Committed gen-m9 cases stay
unchanged. New cases are oracle-filled with excel-graph.

## Acceptance criteria
- [ ] After `make eval`, report.json `argument_coverage` shows `covered == expected` for every parameter of the 129
      functions. Where Excel refuses a workbook holding a kind, the kind is exempted in the catalog with its reason
      instead.
- [ ] Every new cell where ours differs from Excel is fixed in this PR, if it is a one-line coercion fix in the
      function's own module, or listed by case and cell in a functions-stat or functions-criteria task filed with
      `make new-task` (`campaign: semantics` and a `## Visible-parity stage` section for wrong types and errors). The
      new known failures are recorded with `make eval-baseline`, and `make unowned-off ALL=1` lists none of them.
- [ ] Each new Excel rule is in docs/excel-semantics/statistics.md (or criteria.md, arrays.md), citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Covering every function in one PR: if the work exceeds size M, split it by function group and file the rest
  as follow-up tasks with `make new-task` (as T-2861..T-2878 did).
- FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT (T-2425).
- The financial (T-3184), engineering and database argument kinds.
- Fixing numeric residue of new cells (file it as tasks).

## Outcome
- Scope: the value-mode parameter/kind pairs, 335 of the 437 pairs still open on main on 2026-11-04 (the Context's
  573 had shrunk). The 102 array, aggregate and criteria pairs are split out to T-4223 (Out of scope's split rule).
  The generator is docs/architecture/synthetic-generator.md §6.8.
- Three probe cases, probes/t3185-m9-statistical-argument-kinds-a to -c, 335 cells, oracle-filled on excel-graph
  (2026-11-04) with no fill failures. After `make eval`, report.json `argument_coverage` has `covered == expected`
  for every value-mode parameter of the M9 statistical functions (FORECAST.ETS family excluded); what is left is
  array 56, aggregate 23, criteria 22 and aggregate_a 1 (T-4223).
- docs/excel-semantics/statistics.md "M9 statistical argument kinds of a scalar parameter (T-3185)" records the
  rules by kind, citing the cells.
- Ours against Excel: 315 of 335 cells match. The 6 visibly-off cells (b A62, A89, B89, A94, B94; c A69: LINEST,
  LOGEST, GROWTH and TREND given a two-cell `const` or `stats` range) are owned by T-4222; its acceptance criteria
  now name each case beside its cells, so `scripts/new_case_owners.py` passes. The rest are the base calls'
  last-digit residue, not the argument kind's (a A14, A33, A114; b A6, A65, A68, A76, A79, A82, A86, A87, A91, A93,
  A119), out of scope here.
- `make unowned-off ALL=1` lists no t3185 cell (32 unowned cells, all pre-existing on main).
- Follow-ups filed: T-4222 (the 6 visibly-off cells), T-4223 (the 102 array, aggregate and criteria pairs) and
  T-4224 (T-4099's review follow-up).
- Math sidecars: the full `rustsheet-accuracy math --suite probes` run (2026-11-04) wrote none for the t3185 cases,
  so none is committed; `make eval-math-check` is current.
- Eval delta: probes gains 3 cases; 20 new-coverage known failures recorded with `make eval-baseline`
  (probes known failures 4044 -> 4064, 0 fixed, 0 regressions); visible parity allows 6 more cells, all in the new
  cases and owned by T-4222.
