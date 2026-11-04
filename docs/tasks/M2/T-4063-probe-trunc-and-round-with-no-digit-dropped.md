---
id: T-4063
title: "Probe TRUNC and ROUND with no digit dropped at the x87 conversion points"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3931]
---

## Goal
Oracle evidence says which double TRUNC, and ROUND asked for more places than the value's 15 digits, return at the points where Excel's x87 conversion of `m × 10^-n` leaves the nearest double.

## Context
- numbers.md "Decimals reach a double through an x87 multiply by 10^-n": T-3931 applies the model to ROUND and
  ROUNDDOWN (`x87_decimal_to_f64` in crates/rustsheet-engine/src/decimal_round.rs). Two paths are unprobed
  (T-3931 fidelity review):
  - TRUNC goes through `RoundMode::Trunc` and keeps the nearest double, so the engine now separates
    `TRUNC(4910.4E-9,9)` from `ROUNDDOWN(4910.4E-9,9)`, while numbers.md "TRUNC is ROUNDDOWN" says they agree on every probed cell.
  - ROUND with nothing dropped (`ROUND(4.91*10^-6,20)`: num_digits past the 15 significant digits) strips the
    15-digit mantissa's trailing zeros (491 × 10^-8) and runs the model; every probed ROUND cell drops a digit.
- Inputs: the Hits!B inputs of probes-numeric/t3658-round-scaling under TRUNC, and `ROUND(m*10^-n,k)` (a product,
  so the input is the nearest double, not an x87 literal) for k = n+1, 15 and 20 at the same points.

## Acceptance criteria
- [ ] A probes-numeric case with the cells above, oracle-filled (excel-graph), committed with its sidecars.
- [ ] numbers.md records the result for TRUNC and for ROUND with nothing dropped (VERIFIED, or the rule it found); a task for any engine change it needs is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (file a follow-up).
