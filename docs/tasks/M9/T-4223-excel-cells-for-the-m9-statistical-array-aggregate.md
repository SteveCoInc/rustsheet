---
id: T-4223
title: "Excel cells for the M9 statistical array, aggregate and criteria argument kinds T-3185 left (102 parameter/kind pairs)"
milestone: M9
lane: research
status: in-progress
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
- [ ] After `make eval`, report.json `argument_coverage` shows `covered == expected` for every M9 statistical
      parameter except FORECAST.ETS*, or the kind is exempted in the catalog with its reason.
- [ ] Every new visibly-off cell has an owner task; each new rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FORECAST.ETS and its family (T-2425).
