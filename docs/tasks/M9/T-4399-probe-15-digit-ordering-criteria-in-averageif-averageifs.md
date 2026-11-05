---
id: T-4399
title: "Probe 15-digit ordering criteria in AVERAGEIF, AVERAGEIFS, MINIFS, MAXIFS and the D-functions (only COUNTIF, SUMIF, COUNTIFS and SUMIFS have cells)"
milestone: M9
lane: functions-criteria
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AVERAGEIF, AVERAGEIFS, MINIFS, MAXIFS and the D-functions have excel-graph cells at the 15-digit rounding boundary of
the ordering criteria (`<`, `<=`, `>`, `>=`, with `=` and `<>` as controls), so the rule criteria.md states for four
functions is evidenced, or corrected, for every function that shares the matchers.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 8465f6f1, excel-graph sidecars): no failing criteria cell is
  unowned, so this triage read the rules the semantics docs leave unevidenced. criteria.md "Numeric `<`, `<=`, `>`,
  `>=` criteria compare the 15-digit value of the cell" ends: "AVERAGEIF(S), MINIFS/MAXIFS and the D-functions share
  these matchers but have no oracle cell near a rounding boundary yet. Tasks: T-3084, T-3103" (both done).
- Evidence so far: probes/t3072-criteria-near-equal-order (excel-graph, 2026-10-29, 288 criteria cells; numbers.md
  "Criteria ordering operators compare at 15 digits too"). Over one-cell ranges holding `=1+k*2^-52` and `=1-k*2^-53`,
  k = 1, 2, 3, 8, 22 above and 1-4 below count for `"<=1"`, `">=1"` and `"=1"`, not `"<1"` or `">1"`; `1+23*2^-52` and
  `1-5*2^-53` flip all six. COUNTIF, SUMIF, COUNTIFS and SUMIFS agree.
- Ours: the same matchers (`cmp_excel_precision` under the four ordering matchers of functions/criteria.rs, and
  `number_interval` in functions/statistical/ifs_cache.rs). The D-functions build their criteria through
  `build_criteria_with` (functions/database.rs). A scratch case on a release build of 8465f6f1 (A2 `=1+22*2^-52`,
  A3 `=1+23*2^-52`, A4 1, header A1 "v") gives `COUNTIF(A2:A4,">1")` 1, `DCOUNT(A1:A4,"v",C1:C2)` 1 with C2 `">1"`
  and 2 with C2 `"<=1"`,
  `MAXIFS(A2:A4,A2:A4,"<=1")-1` 4.88E-15 (the 22-ulp cell), `AVERAGEIFS(A2:A4,A2:A4,">1")-1` and
  `MINIFS(..,">1")-1` 5.11E-15 (the 23-ulp cell). So ours applies the rule everywhere, UNVERIFIED for these
  functions.
- Excel evidence for the probe values only needs the oracle; no fitting.

Probe (excel-probe skill, excel-graph, tag M9), probes/t4399-criteria-near-equal-others, reusing t3072's value column
(rows 1-12: `=1+k*2^-52` for k = 1, 2, 3, 8, 22, 23 and `=1-k*2^-53` for k = 1-5, and 1):
- AVERAGEIF(B1:B12, crit, B1:B12) and AVERAGEIFS, MINIFS, MAXIFS over the same column, for each of the six criteria
  `"<1"`, `"<=1"`, `">1"`, `">=1"`, `"=1"`, `"<>1"`, both over the whole column and over each one-cell range (as
  t3072 does, so each cell's membership shows), with `-1` after the aggregate so the picked value's ulp shows in the
  digits;
- DCOUNT, DSUM, DAVERAGE, DMIN and DMAX over the column with a header, and a criteria range holding each of the six
  criteria texts, one criteria range per row.

## Acceptance criteria
- [x] probes/t4399-criteria-near-equal-others is committed with excel-graph oracle values.
- [x] Every cell passes, or each cell ours gets wrong is fixed here (a small change in the shared matcher's caller in
      functions/criteria.rs, functions/statistical/if_ifs.rs or functions/database.rs, with a unit test) or owned by
      a new functions-criteria task filed with `make new-task` that names the case and cells
      (`campaign: semantics`, with a `## Visible-parity stage` section).
- [x] criteria.md's "no oracle cell near a rounding boundary yet" sentence is replaced by the probed result, citing
      the cells; numbers.md's entry names the new functions.
- [x] `make eval-baseline` records no regression; `make check` green; reviewers approve.

## Out of scope
- Text ordering criteria and dotted dates (criteria.md, their own entries).
- The `=` and `<>` rule of the operators outside criteria (numbers.md "Comparison tolerance").

## Outcome
Probes/t4399-criteria-near-equal-others (379 cells, excel-graph) added; every cell matches ours, so no engine change was needed. criteria.md and numbers.md now cite it. Eval delta: +379 bit-exact probe cells, no regressions.
