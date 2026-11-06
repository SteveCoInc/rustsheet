---
id: T-4510
title: "VDB from a fractional start where straight line wins at period 0 (Gate A T-4111)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VDB in the switching mode returns Excel's value from a fractional start when straight line already wins at period 0,
so the six Gate A points below pass and their known-failures lines go.

## Context
- Found by T-4111's Gate A table (excel-graph): where `(cost - salvage)/life > cost·factor/life` (a factor below
  `1 - salvage/cost`) and `salvage <= cost`, Excel charges `(cost - salvage)/life` for every period of the window,
  the start's fraction included: `VDB(121503138647.778,3864354372.11506,4,0.5,4,0.15833054798759,FALSE)` is
  102933936241.20506, `3.5·(cost - salvage)/4`. The engine (`vdb_switching_between`,
  functions/financial/financial_depreciation.rs) first charges the fraction at the declining rate and then switches
  at the fraction's grid point, 115234076960.1236 (10.7% off). Points: accuracy/VDB/s1!Calc!A158, A183, A187, A188,
  A189, A200 (the rule gives Excel's value to the bit or within 1 ulp on all six, and on the whole-start A119).
- financial.md "VDB charge order" records the rule (T-4111); the T-4111 definition (defs/depreciation.py,
  `_vdb_switching`) encodes it.
- Check that the rule keeps every probes/fin-vdb-* cell (161 switching-mode cells) and the gen-m9 VDB cells.

## Acceptance criteria
- [ ] The six points pass Gate A; their lines in eval/accuracy/known-failures.tsv are removed.
- [ ] eval-check: 0 regressions in the VDB cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `no_switch` TRUE mode, which this rule does not touch.
