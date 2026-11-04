---
id: T-3141
title: "CUMPRINC where v^nper underflows: the last bits of the balance-drop principal (T-3067's seven regressed near-1E15 cells and the underflow-ends misses)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3067]
---

## Goal
Type 0 CUMPRINC (and the CUMIPMT built on it) is Excel's to the bit where `v^nper` underflows, or the forms tried are
recorded in financial.md.

## Context
T-3067 run 5 found Excel's last-m-period principal does not depend on nper once `v^nper` underflows
(probes/fin-cumprinc-underflow-ends: nper 1E6, 1E10, 1E12 and 1E15 give the same doubles) and took the balance drop
`-(pv·(v^(nper-end)·(1-v^m)))` on Excel's EXP, Kahan's expm1 and `growth_log` there (`underflowed_principal` in
functions/financial/cumulative.rs; financial.md "T-3067 notes"). It is exact on 70 of the 108 underflowed principals
probed and within an ulp of the rest. Decision 0160 let it move these 7 cells off the bit; this task owns them:
- probes/fin-cumprinc-near-1e15 A7 (+1 ulp), B1, B4, B9, C10 (-1 ulp each), C6 (+1 ulp);
- probes/fin-cumprinc-near-1e15 E7 `CUMIPMT(0.01,1E15,100000,1E15-6,1E15,0)` (16 ulps, visibly off: A7's ulp through
  the payments less the principal).
- probes/fin-cumprinc-near-1e15 E2, E3, E4 `CUMIPMT(0.01,1E15,100000,1E15-k,1E15,0)` for k = 1, 2, 3 (Excel
  -29.60494069208835, -59.01479276444434, -98.03444828162765; ours -29.604940692088576, -59.01479276444388,
  -98.0344482816281: visibly off, the same cancellation of CUMPRINC's 1-ulp misses at A2, A3, A4; no open task named them,
  gap triage 2026-10-29).
Also owned here (gap triage 2026-10-30, each within an ulp, no open task named them): probes/fin-cumprinc-near-1e15
B10, B12, C1, C9 (type 0, e.g. `CUMPRINC(0.1,1E15,100000,1E15-0,1E15,0)` Excel -9090.909090909092, ours
-9090.90909090909) and F1, F2, F3, F4, F5, F6, F7, F9, F11, F12 (type 1, e.g. `CUMPRINC(0.01,1E15,100000,1E15-1,1E15,1)`
Excel -1950.886197334566, ours -1950.8861973345654).
Start with E7, the one visibly-off cell. Also open: the underflow-ends misses in A-D, G, H (each within an ulp), fin-cumipmt-overflow-and-forms A10 and the
type 1 ranges (column J, fin-cumipmt-period-bound A18), which still take the closed form. The implied L per cell is not
one biased log (the windows for m = 2 and m = 8 at 0.01 do not overlap), so the error is in the operation order; try
x87-register intermediates (`Register`) and the twice-rounded operators of numbers.md before more double orderings.
Decision 0056 applies (two attempts, then time-box).

## Acceptance criteria
- [x] The 7 cells above pass (`make eval-baseline`) with no new known failures, or every form tried is in financial.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Ranges where `v^nper` does not underflow (T-3067, T-1374).

## Outcome
Fixed, not time-boxed. Where `1 - v^nper` is 1, Excel's principal over a short range is one core for both types,
`C = pv·((1-v^m)·v^(nper-end+1))`: type 1 is `-C`, type 0 `-(C·(1+r))`, with `1-v^m` PMT's `discount_complement`
(Kahan's expm1 below m·L = 1, `1 - EXP` from 1 on), the power Excel's EXP of `-(nper-end+1)·L` and `growth_log`'s L.
T-3067's form folded the `1+r` into the power, which is what left the last bits off. `underflowed_principal` now
takes both types (not a type 1 range from period 1) and comes before every other short form, at any leading exponent.
Evidence and the search are in financial.md "Where `v^nper` underflows, the principal is one core for both types".
- Structural model (decision 0188 R4: hold-out and fresh sweep, no `FITTED` constant; the 1.0 cut is T-3014's, reused):
  fit set the 132 principals of fin-cumprinc-near-1e15 and fin-cumprinc-underflow-ends; hold-out the new
  probes/fin-cumprinc-underflow-type1 (72 cells, excel-graph 2026-11-04), split before scoring; the enumeration of
  every expression tree over the factors found the core, which is exact on 132/132 and 72/72. Fresh sweep after the
  freeze: the new probes/fin-cumprinc-underflow-fresh (200 cells: 5 rates, own nper and pv, CUMPRINC and CUMIPMT of both
  types): 98/100 principals with Kahan's expm1 throughout; the 2 misses (I5, J5, m·L 1.40) are `1 - EXP`'s, which PMT's
  existing switch at 1 gives (Kahan right at 0.98, I4), so 100/100 and all 200 cells to the bit. No keep-matched branch.
- Also found: Excel's type 1 CUMPRINC there is its type 0 over `1+r` in doubles on 58 of 60 probed pairs (the other 2
  are `C·(1+r)/(1+r) ≠ C`), recorded in financial.md.
- Eval delta: 71 known failures fixed, 0 regressions; probes 4417 -> 4349 (68: fin-cumprinc-near-1e15 27, including
  A7, B1, B4, B9, C6, C10, E2-E4, E7, F1-F12 of this task; fin-cumprinc-underflow-ends 39; fin-cumipmt-overflow-and-forms
  A10; fin-cumipmt-period-bound A18), gen-m9 1939 -> 1936 (67a4c03afd63f988 A208, b73c6cc96e093f03 A111,
  d0854366830dcdee A98: integer rates). The two new cases (272 cells) are bit-exact. fin-cumprinc-underflow-ends F
  (nper 1000, `v^nper` not underflowed; within an ulp) stays: out of scope here.
- Unit test: `an_underflowed_principal_is_one_core_for_both_types` (test_fn_cumulative_lift.rs).
