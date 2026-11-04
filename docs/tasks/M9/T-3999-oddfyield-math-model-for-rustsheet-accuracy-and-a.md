---
id: T-3999
title: "ODDFYIELD math model for rustsheet-accuracy and a Gate A table (long first periods)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math` can compute ODDFYIELD to 50 digits (ODDFPRICE's root from the schedule's exact day counts),
and Gate A has an ODDFYIELD table, so ODDFYIELD fits (decision 0188 R2) cite the math instead of a scratch script.

## Context
- Follow-up of T-3031 (review round 1, reviewer-excel-fidelity): its Gate A evidence for probes-numeric/
  t3031-r4-oddfyield-long-first S!A61 and S!A70 is a scratch mpmath root of ODDFPRICE(y) = price over the engine's
  `OddFirstSchedule` fields (Excel -3.7e-11 and +4.9e-11 relative of the root, ours -9.0e-11 and +1.2e-10), and that
  case has no math sidecar because `rustsheet-accuracy math` has no ODDFYIELD model.
- ODDFPRICE's schedule is in `functions/financial/financial_bonds.rs` (`OddFirstSchedule::new`, short and long first
  periods; financial.md "ODDFPRICE"); the math model takes the day counts as exact rationals, not doubles.
- Excel stops short of the root by 3e-12 to 3e-8 relative on the 73 long-first cells T-3031 lists (financial.md
  "ODDFYIELD stops short of ODDFPRICE's root"), so most points will be Excel-vs-math differences inside the 1e-9
  envelope or R3 candidates; record them, do not change the engine here.

## Acceptance criteria
- [ ] An ODDFYIELD model in `rustsheet-accuracy math` (50 digits; short and long first periods, every basis), with
      a unit test against two hand-checked roots (T-3031's S!A61, S!A70).
- [ ] A Gate A table accuracy/ODDFYIELD/s1 (excel-graph) and the t3031 case's math sidecar; `rustsheet-eval accuracy`
      green with any outside-envelope points owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing ODDFYIELD's solver (T-3988 owns the month-end cells).
