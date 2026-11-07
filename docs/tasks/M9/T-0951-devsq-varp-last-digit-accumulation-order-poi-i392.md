---
id: T-0951
title: "DEVSQ/VARP last-digit accumulation order (poi I392, K1456, E392, F392, J392)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
DEVSQ and VARP reproduce Excel's last digit on poi EverythingTests!I392, K1456, E392, F392 and J392.

## Context
Left over from T-0945. `DEVSQ("1","2",2)` expects 0.6666666666666666 (engine 0.6666666666666661 with sumsq - sum^2/n; a two-pass mean/deviation sum gives the expected value, but VARP(TRUE,FALSE,TRUE) expects 0.2222222222222222 and two-pass gives ...24). Numeric fitting: find the accumulation formula that satisfies all five cells and the currently passing ones.

## Acceptance criteria
- [ ] The five cells leave eval/baseline/poi.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- I392's visible-parity target belongs to T-1749 since the 2026-10-28 triage (it became visibly off, 5E-16 from Excel); this task keeps its last digit and the VARP cells.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Cells added by gap triage (2026-10-31m)
`make eval` on origin/main 900fbc1bf (excel-graph). An ownership check over every failing Excel cell
(`scripts/exit_owners.py`'s `owned` rule over the open task files of origin/main and of every `origin/agent/*` branch
pushed since 2026-10-29) found these cells named by no open task. All are within 15 significant digits of Excel (Phase
1b), so they join this deferred owner of their kernel. `DEVSQ(C1:C6,5)` on the stat argument-modes probe, 2 ulp (Excel
8.666666666666668, ours ...664).
- probes/stat-argument-modes.case.json (1): Sheet1!H5

Gap triage 2026-11-03 (`make eval` on origin/main b9dac1c15 with poi fetched): the poi cells of this task's title
were named without the workbook, so the ownership check saw E392, F392 and J392 as unowned (all 1 ulp; I392 is
T-1749's visible target, K1456 now passes). Excel against ours: E392 `DEVSQ(B7:B9,C7:C9,B15:B17)` 5.837134000085715,
ours ...714; F392 `DEVSQ(B13:D14)` 749730712528.7554, ours ...7555; J392 `DEVSQ(F10,I7)` 4.499970000049999, ours
4.49997000005.
- poi/FormulaEvalTestData_Copy.xlsx (3): EverythingTests!E392, F392, J392
