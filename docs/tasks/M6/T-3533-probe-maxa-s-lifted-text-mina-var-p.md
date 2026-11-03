---
id: T-3533
title: "Probe MAXA's lifted text, MINA, VAR.P, STDEV.P, the VARA family and legacy VAR over a lifted OFFSET (T-2824 follow-up)"
milestone: M6
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2824]
---

## Goal
Excel's results are known for the aggregates T-2824 left on the value view, or read without evidence, over an array of
references (a lifted OFFSET), so each can follow Excel.

## Context
T-2824 made STDEV.S, VAR.S, SUMSQ, AVERAGEA and MAXA read each element of a lifted OFFSET by its top-left cell, on
probes/t2065-lift-* and probes/t2816-lift-* (docs/excel-semantics/lookup.md "Statistical aggregates over a lifted
OFFSET"). These are still open:
- MAXA reads that cell as an array element, which skips text. No probe tells skipped text from text read as 0:
  `MAXA(OFFSET(A4,{0;1},0))` with A4 "5" and A5 -3 gives 0 if text counts 0, and -3 if it is skipped.
- MINA, VAR.P, STDEV.P, VARA, VARPA, STDEVA and STDEVPA keep the value view (`#VALUE!`) and are unprobed.
- Legacy VAR shares VAR.S's reading, UNVERIFIED. Legacy STDEV is verified by probes/t2577-indirect-ref-array Calc!A157.
Write one formula per workbook, in E1, on the t2816 data (probes/t2816-lift-*).

## Acceptance criteria
- [x] Probe cases filled on excel-graph for each function above, with single-cell, multi-cell, text, blank and TRUE
      elements; the finding is in lookup.md; the engine follow-up is filed, or the change is made if it is S-size.
- [x] `make check` green; reviewers approve.

## Out of scope
- MEDIAN, LARGE and SMALL (verified `#VALUE!`, lookup.md).

## Outcome
- 51 probe cases probes/t3533-lift-*, one formula per workbook in E1 on the t2816 data plus A8 = -3, filled from
  excel-graph (generator docs/research/t3533/mkcases.py): VAR.P, STDEV.P, VARA, VARPA, STDEVA, STDEVPA and legacy VAR
  over number, multi-cell, text, blank and TRUE elements (three elements, so skipped, 0 and 5 differ), and MAXA and
  MINA over the same with a -3 or 7 companion.
- Finding (lookup.md "Statistical aggregates over a lifted OFFSET"): every one reads each element's top-left cell and
  skips a text, blank or TRUE element, the A-variants included; MAXA's text is skipped, not 0; legacy VAR is VAR.S's.
- Engine (S-size, done here): VAR.P and STDEV.P read the grid by `GridReading::TopLeft` (`variance_of_values` lost its
  now-constant `grids` parameter and absorbed `variance_of_values_a` behind an `a_rules` flag, since the two
  bodies became identical; MINA and MAXA share `extremum_a`), MINA and `VARA_ARGS` (VARA, VARPA, STDEVA, STDEVPA) by
  `GridReading::TopLeftElements`. Unit test `fn_offset_lifted_into_the_statistical_aggregates_reads_top_left_cells`
  extended; the value-view test now covers MEDIAN only.
- Eval delta: the 51 new probes all pass (16 MAXA/VAR passed before the change, 35 fixed by it); regressions 0.
- Follow-up filed: T-3968 (T-3953 review MINORs, unrelated to this task's lane, carried in this PR per work-loop step 5).
