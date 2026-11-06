---
id: T-4537
title: "MIRR is #DIV/0! when a finance power overflows under a later negative flow"
milestone: M9
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MIRR gives Excel's error code when its finance power overflows and the flows hold a positive flow and a negative flow
after the first, so the Gate A known failure accuracy/MIRR/s1 A125 goes.

## Context
- financial.md "MIRR at finance 1E+200 over five flows with a negative flow at index 2 is #DIV/0! (T-4116)": found by the T-4116
  Gate A table. `MIRR({-3496.38522311601,97.9347197399849,-81.6919783865016,180.891974550287,218.629532704649},1E+200,0.00166838654471219)`
  is `#DIV/0!` in Excel (accuracy/MIRR/s1 A125); the engine gives `#NUM!`, because it picks the finance overflow's error
  from whether a positive flow exists (`MIRR({-100,50,80},1E+200,0.1)` is `#NUM!`, probes/fin-mirr-edges A15).
- What separates the two is unprobed: the negative flow after the first (A125 has one, A15 and A24 none), or the index
  at which the finance power first overflows against the negative flows' indices (A125's first overflowing power,
  (1+fr)^2, is the one its negative flow at index 2 is divided by). Probe it first (excel-probe): finance 1E+200 with a
  negative flow at index 1 only (whose power, 1E+200, is finite) and at index 2 only, beside positive flows; finance
  1E+300 with a negative flow at index 1; and a negative flow at index 3 with powers overflowing from index 2.
- Code: `fn_mirr` in `crates/rustsheet-engine/src/functions/financial/mirr.rs` (the overflow branch after the reinvest
  -1 check).
- Known failures: the `eval/accuracy/known-failures.tsv` line owned by T-4537 (remove it when it passes).

## Acceptance criteria
- [ ] The rule is probed and recorded in the semantics entry, and `fn_mirr` follows it.
- [ ] accuracy/MIRR/s1 A125 passes and its known-failure line is gone; the fin-mirr-edges cells still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MIRR's power underflow and the reinvest power's errors (financial.md "MIRR's evaluation order", T-1995).

## Outcome
Probed (probes/fin-mirr-overflow-negative, excel-graph): a negative flow divided by an infinite finance power is
`#DIV/0!`; negative flows before the first overflowing power leave `#NUM!`. `mirr_of` returns `#DIV/0!` in that case;
semantics entry updated; accuracy/MIRR/s1 A125 passes and its known-failure line is removed.
