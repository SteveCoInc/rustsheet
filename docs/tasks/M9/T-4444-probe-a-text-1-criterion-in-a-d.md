---
id: T-4444
title: "Probe a text =1 criterion in a D-function criteria range near 1; tidy T-4399's tag and numbers.md evidence line"
milestone: M9
lane: functions-criteria
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4399]
---

## Goal
A D-function criteria range holding the text `=1` (not the number 1) is probed against Excel near 1 at the 15-digit boundary, and T-4399's case tag and numbers.md evidence line read correctly.

## Context
MINOR findings of T-4399's review (PR #3431, all reviewers approved 9bd0129):
- eval/suites/probes/cases/t4399-criteria-near-equal-others.case.json BF2 is the spec cell `"=1"`, a formula, so
  Excel stores the number 1 and BF4:BF8 test a numeric criterion (criteria.md says so). The text form of the criterion
  `=1` in a criteria range is unprobed: write the cell as text (`'=1`, or check how spec2xlsx stores a literal text
  starting with `=`) against t3072's boundary values (1+23·2^-52, 1-5·2^-53).
- The case is tagged `M2`; the task asked for M9 (re-tag it).
- docs/excel-semantics/numbers.md (the entry citing t4399 and t3072): the t3072 column layout ("rows 1-12 ... E:J
  COUNTIF ...") follows the t4399 citation, so it reads as if it described both probes; start it with t3072's name.

## Acceptance criteria
- [ ] A probe case with an excel-graph sidecar covers DSUM/DCOUNT/DAVERAGE with a text `=1` criterion over the
      boundary values; each cell passes, or the rule is recorded in criteria.md and its cells are owned by a task.
- [ ] The t4399 case's tag and the numbers.md evidence line are fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/t4444-dfunction-text-equal-one (52 cells, excel-graph): a criteria cell holding the text `=1` or `=1.0`
gives the same DCOUNT/DSUM/DAVERAGE/DMIN/DMAX results as the numeric `=1` control; all 52 cells pass, no engine change.
Re-tagged t4399 M2 -> M9; reworded the numbers.md t3072/t4399 evidence line; recorded the rule in criteria.md.
Eval delta: +52 passing cells (probes), no regressions.
