---
id: T-4531
title: "PERCENTRANK family: probe how a huge significance's #DIV/0! orders against an x outside the data and one-value data, and the direction of an exact 16th-digit tie in the 15-digit cut, and match"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The two UNVERIFIED parts of docs/excel-semantics/statistics.md "PERCENTRANK: a huge significance
is #DIV/0!; truncation does not lose a digit at a cut (T-2256)" have Excel evidence and the engine
matches it: where the significance-overflow `#DIV/0!` sits in PERCENTRANK's check order, and which
way the 15-significant-digit cut of rank × 10^significance breaks an exact tie.

## Context
- VERIFIED: a significance of 309 or more is `#DIV/0!`, 308 computes (probes/t2473-percentrank-
  significance A1-C15), and a significance below 1 is `#NUM!`; the rank is cut to 15 significant
  digits, then floored (t2473 D1-D24). Every probed `#DIV/0!` cell has x inside data of 3 or more
  values (`PERCENTRANK({1,2,3,4},2.5,309)` and similar).
- UNVERIFIED, no open owner (the entry's own words): "how the `#DIV/0!` orders against an x outside
  the data (`#N/A`) and a single value (1); ours checks it after both. The direction of an exact
  16th-digit tie in the cut (ours away from zero, as FLOOR's) is unobserved, as no probed rank
  reaches one." Ours: `percentrank_of` (functions/statistical/percentile.rs) and `decimal_round::cut_to_15_digits`
  (crates/rustsheet-engine/src/decimal_round.rs).
- Opus: the tie half needs ranks constructed so that rank × 10^significance lands exactly on a
  16th-digit half (a rational k/n with a power-of-two n and the right significance), checked
  against mpmath before committing; this is constructed evidence, not a fit.
- Neighbours: T-2605 (sonnet: a single boolean cell and a bad significance on empty data, which is
  the `#N/A`-vs-significance order for empty data, not this one), T-4093 (Gate A tables for the
  family). Keep the cells disjoint from T-2605's.
- Use the excel-probe skill (excel-graph oracle).

## Acceptance criteria
- [ ] A committed probe case with an excel-graph sidecar holds PERCENTRANK, PERCENTRANK.INC and
      PERCENTRANK.EXC with significance 309, 400 and 2147483648 against: x below and above the data,
      one-value data with x equal to and not equal to the value, and a `#NUM!` significance (0.5)
      beside an outside x, for comparison.
- [ ] The same case holds at least six ranks whose rank × 10^significance is an exact 16th-digit
      tie (both signs of rounding direction distinguishable), with the mpmath values in the case
      description or a math sidecar.
- [ ] Every new cell passes, or each remaining failure is owned by a filed task named in
      statistics.md.
- [ ] statistics.md's entry replaces both UNVERIFIED sentences with the probed rule and cites the
      cells.
- [ ] No new known-failure lines in eval/baseline/probes.txt or eval/baseline/gen-m9.txt other than
      the new case's owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Single boolean cells and empty data (T-2605).
- PERCENTRANK's interpolation digits away from a tie (T-4093's Gate A tables).
