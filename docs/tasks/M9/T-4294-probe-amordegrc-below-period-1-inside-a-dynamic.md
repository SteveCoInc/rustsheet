---
id: T-4294
title: "Probe AMORDEGRC below period 1 inside a dynamic-array formula, and in a LAMBDA-running call (BYCOL, GROUPBY, a defined-name LAMBDA)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for two open corners of financial.md "AMORDEGRC below period 1", and the engine matching it.

## Context
- From the Excel-fidelity review of T-3393 (PR #3284, MINOR findings).
- **A reading anchor.** A dynamic-array anchor that reads the left-over value is evaluated before the Excel-order
  chain, so it reads the value too early. Example: `=AMORDEGRC(10000,39000,39400,500,0.3,0.3,0)+SEQUENCE(1,2)` in A1
  above a 485 charge in A2. The rule predicts 485 and ours gives 0. The relevant code is `run_pass` and
  `AnchorAmor` (excel_order.rs).
- **LAMBDA-running calls.** Ours empties the value after every LAMBDA it runs (`call_prepared`). That covers BYCOL,
  GROUPBY and PIVOTBY with a function argument, and a defined-name LAMBDA, none of which is probed. Only
  `LAMBDA(x,x)(1)`, MAP, BYROW, SCAN, REDUCE and MAKEARRAY are probed.

## Acceptance criteria
- [ ] Probe workbooks, one question each, filled on excel-graph:
  - the reading anchor above;
  - the same anchor with the fraction after the SEQUENCE;
  - BYCOL, GROUPBY with `SUM` and with `LAMBDA`;
  - a defined-name LAMBDA call, between the fraction and the 485 charge.
- [ ] financial.md records the results. The engine matches them, or a task owns every visibly-off cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- WEEKNUM's state (T-4289).
