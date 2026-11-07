---
id: T-5051
title: "Score a spilled kept subnormal that Excel for the web saves as 0 (decision 0042 companions in array context)"
milestone: M11
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A spilled element whose Excel value is a kept subnormal, saved by Excel for the web as 0, scores as decision 0042 scores a scalar one, or the cell is recorded as an accepted known failure by a decision.

## Context
Found by T-4766. Decision 0042 (`graph_hides_subnormal`) passes an excel-graph 0 against an engine subnormal only when the case
has a FALSE companion `=(<formula>)=0`. For an array formula that companion is evaluated in array context, where Excel reads a
number below 2^-1026 as equal to 0 (docs/excel-semantics/numbers.md "Worksheet EXP (T-1931)", T-4766), so it is TRUE and the
rule cannot apply. A scalar companion such as `=(INDEX(<formula>,1))=0` is FALSE in Excel
(probes/t4766-array-subnormal-compare A5) and would carry the evidence per element. Off cells this task owns:
probes/subnormal-saved-negative-and-array S!A3 and probes/t4766-array-subnormal-compare S!A30 (`=EXP({-745,-708})`, Excel's
saved 0 against the engine's 2^-1074). Changing the harness rule needs a decision record (decision 0042 is the rule).

## Acceptance criteria
- [ ] A decision record extends or declines decision 0042 for spilled elements; the two cells pass or are accepted known
      failures under it.

## Out of scope
- Desktop Excel's save (M14 is out of scope, decision 0266).
