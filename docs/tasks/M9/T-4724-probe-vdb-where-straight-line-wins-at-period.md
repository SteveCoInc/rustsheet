---
id: T-4724
title: "Probe VDB where straight line wins at period 0: a whole start with a fractional end, and factor 0 from a fractional start"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence on VDB's switching mode where straight line already wins at period 0
(`(cost - salvage)/life > cost·factor/life`, salvage at most cost), in the two shapes T-4510 left unprobed, and the
engine matching it.

## Context
- T-4510 charges `(end - start)·sl` (sl = `(cost - salvage)/life` rounded first) from a fractional start in this
  regime (financial.md "VDB charge order", "Straight line from period 0"; accuracy/VDB/s1 A158, A183, A187, A188, A189,
  A200 to the bit). A whole start keeps the schedule (`vdb_switching_between`,
  functions/financial/financial_depreciation.rs), which sums a prorated last period first and adds the whole
  straight-line periods as one product `n·sl`.
- Unprobed 1: a whole start with a fractional end, e.g. `VDB(86661,45,37,0,5.5,0.2)`. There `(end - start)·sl` and the
  schedule's `share·sl + n·sl` differ in the last bits on about 14% of random draws (T-4510 excel-fidelity review).
- Unprobed 2: factor 0 from a fractional start, e.g. `VDB(1000,100,10,0.5,3,0)`. The T-4111 rule gives
  `(end - start)·(cost - salvage)/life`, while the fractional grid would charge about `(cost - salvage)/(life - f)`. The
  engine takes the T-4111 rule today. probes/t2492-vdb-ddb-arguments has factor 0 only with whole starts.

## Acceptance criteria
- [ ] An excel-graph probe (predictions committed first) covering both shapes, with a few dozen draws each, is filled.
- [ ] The engine matches its cells (the early return in `vdb_switching_between` extended or narrowed to fit), and
      financial.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `no_switch` TRUE mode.
