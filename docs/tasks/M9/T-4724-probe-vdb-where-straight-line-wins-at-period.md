---
id: T-4724
title: "Probe VDB where straight line wins at period 0: a whole start with a fractional end, and factor 0 from a fractional start"
milestone: M9
lane: functions-financial
status: done
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
- [x] An excel-graph probe (predictions committed first) covering both shapes, with a few dozen draws each, is filled.
- [x] The engine matches its cells (the early return in `vdb_switching_between` extended or narrowed to fit), and
      financial.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `no_switch` TRUE mode.

## Outcome
- Probe: probes/t4724-vdb-straight-from-zero (excel-graph, 2026-11-07). The predictions, with the engine's value
  before the change, were committed before the fill (0f2bf002d, docs/research/t4724).
  - **W** (30 rows, a whole start with a fractional end, straight line winning at period 0): Excel is the single
    product `(end - start)·sl` on all 30. The schedule order `share·sl + n·sl` misses all 30, by up to 432 ulp. The
    engine matched 27 (W A7, A15 and A20 were off).
  - **F** (20 rows, factor 0 from a fractional start): Excel is T-4111's `(end - start)·(c - s)/life` on all 20,
    which the engine already gave. The fractional grid's `/(life - f)` misses all 20.
- Code: `vdb_switching_between` (functions/financial/financial_depreciation.rs) now takes the early return
  `(end - start)·sl` for every start, not only a fractional one. This is the form T-4510 first proposed. No constant.
- financial.md "VDB charge order" records both answers. The unit test `fn_vdb_straight_line_from_period_zero_bits`
  gains W A7.
- Eval delta: the new probe passes all 50 cells, and eval-gates shows 0 regressions (VDB/s1 A119 and gen-m9 A334 stay
  exact).
