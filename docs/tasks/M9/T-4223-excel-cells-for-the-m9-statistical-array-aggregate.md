---
id: T-4223
title: "Excel cells for the M9 statistical array, aggregate and criteria argument kinds T-3185 left (102 parameter/kind pairs)"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every array, aggregate and criteria parameter of the M9 statistical functions has an Excel cell of each argument kind
that report.json `argument_coverage` expects, the part of T-3185's goal that T-3185 left.

## Context
- T-3185 covered the 335 value-mode pairs (probes/t3185-m9-statistical-argument-kinds-a to -c). The 102 pairs left,
  on main after T-3185, are array parameters (CHISQ.TEST, CHITEST, F.TEST, FTEST, T.TEST, TTEST, Z.TEST, ZTEST,
  LINEST, LOGEST, GROWTH, TREND), aggregate ones (AVEDEV, DEVSQ, HARMEAN, KURT, MEDIAN, MODE*, SKEW*, STDEV*, VAR*,
  VARPA) and the criteria of AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS.
- T-3185's Context gives the shapes that need care: an array kind both as a direct argument and as an array element.
- The generator is the one T-3185 used (docs/architecture/synthetic-generator.md, its section on hand-written
  argument-kind cases): one valid base call per function with one argument swapped for the kind.

## Acceptance criteria
- [x] After `make eval`, report.json `argument_coverage` shows `covered == expected` for every M9 statistical
      parameter except FORECAST.ETS*, or the kind is exempted in the catalog with its reason.
- [x] Every new visibly-off cell has an owner task; each new rule is in docs/excel-semantics/statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- FORECAST.ETS and its family (T-2425).

## Outcome
- Scope: the 100 array, aggregate and criteria parameter/kind pairs still open on main on 2026-11-06. The Context's 102
  had shrunk; the rest of the FORECAST.ETS family is out of scope.
- Probe cases: probes/t4223-m9-statistical-array-kinds-a and -b, 100 cells, oracle-filled on excel-graph
  (2026-11-06) with no fill failures. They use T-3185's forms (docs/architecture/synthetic-generator.md §6.8). An
  aggregate takes the kind as its first argument, beside `Data!A1:A5`.
- Coverage: `rustsheet-eval run --suite probes --filter t4223` sees every one of the 100 targeted classes. With
  T-3185's cases, `argument_coverage` reaches `covered == expected` for every M9 statistical parameter outside the
  FORECAST.ETS family.
- statistics.md "M9 statistical argument kinds of an array, aggregate or criteria parameter (T-4223)" records the
  rules by kind and cites the cells. One of them: MODE and MODE.SNGL skip a direct `""`, where every other aggregate
  is `#VALUE!`.
- Engine: `ZTEST("3",3)` (b A48) was `#VALUE!` in ours and is `#DIV/0!` in Excel. `z_test.rs` now reads direct
  numeric text as one value, like a direct boolean. Unit test: `direct_numeric_text_is_one_value`.
- Ours against Excel: 100 of 100 cells match, so there are no visibly-off cells and no owners to file.
- Eval delta: probes gains 2 cases, 100 new passing cells, 0 known failures, 0 regressions.
- Follow-ups filed: T-4451 (shared spec2xlsx test helper, PR #3472's design review MINOR, riding here).
