---
id: T-3141
title: "CUMPRINC where v^nper underflows: the last bits of the balance-drop principal (T-3067's seven regressed near-1E15 cells and the underflow-ends misses)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3067, T-4059]
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

## Blocked because
Blocked because: the R4.2 fresh sweep and Gate A envelope need a CUMPRINC Gate A table, which does not exist (T-4059).
Round 1 of review (PR #3105, its first head): design and security approved; Excel fidelity requested changes on the
decision 0188 R4 guards. Round 2 (this head) answers what it can without the table: the fit lab's hold-out split, the
form kept below `GEOMETRIC_MIN_LEAD` (its evidenced domain), and the cut's CUMPRINC evidence (25 cells, m·L 0.36..1.40,
bracket 0.9783..1.3976) noted on `PAYMENT_EXPM1_TO`. Its `FITTED(T-3141, ...)` tag raises the quality/fitted.json lock,
which needs a decision record with the full R4 evidence (like decision 0261), so the tag waits for the sweep.
fin-cumprinc-underflow-fresh served as fit data (its I5 and J5 set the expm1 switch), so it is no R4.2 sweep. Still open:
the `r4-T-3141` batch, filled after the freeze, with its envelope count and every point outside it listed. Resume once
T-4059 lands: record that result here, re-run the full panel, and enqueue. The kernel is unchanged since the freeze.

## Outcome
Where `1 - v^nper` is 1, Excel's principal over a short range is one core for both types,
`C = pv·((1-v^m)·v^(nper-end+1))`: type 1 is `-C`, type 0 `-(C·(1+r))`, with `1-v^m` PMT's `discount_complement`
(Kahan's expm1 below m·L = 1, `1 - EXP` from 1 on), the power Excel's EXP of `-(nper-end+1)·L` and `growth_log`'s L.
T-3067's form folded the `1+r` into the power, which is what left the last bits off. `underflowed_principal` now
takes both types (not a type 1 range from period 1) and comes first of the short forms, but only below
`GEOMETRIC_MIN_LEAD` (0.3), as T-3067's form did. Evidence and the search are in financial.md "Where `v^nper`
underflows, the principal is one core for both types".
- Structural model (decision 0188 R4, kind (c)): found by enumerating every expression tree over the factors, with the
  132 principals of fin-cumprinc-near-1e15 and fin-cumprinc-underflow-ends as the fit set. New probes (excel-graph
  2026-11-04): fin-cumprinc-underflow-type1 (72 cells) and fin-cumprinc-underflow-fresh (200 cells). The frozen core
  was 98/100 on the fresh principals; its misses I5 and J5 (m·L 1.40) set the expm1 switch (PMT's cut, reused), so that
  case is fit data too.
- R4.1, the fit lab's split (`fitlab.holdout.is_holdout`, margin 5 points, the default), over all 304 underflowed
  principals of the four cases:

  | candidate | train exact / 4 ulp / 15 digits | test exact / 4 ulp / 15 digits |
  |---|---|---|
  | the core (kernel) | 216 / 216 / 216 of 216 | 88 / 88 / 88 of 88 |
  | correctly rounded | 65 / 216 / 216 of 216 | 33 / 88 / 88 of 88 |

  Accepted: test rate 100% equals the train rate, and it beats correctly rounded on test.
- R4.2 and R2: open, T-4059 (see Blocked because). R4.3: open with them. Once the sweep is in, add
  `FITTED(T-3141, 25 cells, CUMPRINC underflowed m*L 0.36..1.40; cut bracketed 0.9783..1.3976)` on `PAYMENT_EXPM1_TO`,
  run `scripts/check_fitted.py --update`, and write the lock's decision record (`make new-decision`). No keep-matched
  branch.
- Also found: Excel's type 1 CUMPRINC there is its type 0 over `1+r` in doubles on 58 of 60 probed pairs (the other 2
  are `C·(1+r)/(1+r) ≠ C`), recorded in financial.md.
- Eval delta: 68 known failures fixed, 0 regressions; probes 4417 -> 4357 (68 fixed: fin-cumprinc-near-1e15 27,
  including A7, B1, B4, B9, C6, C10, E2-E4, E7 and F1-F12 of this task; fin-cumprinc-underflow-ends 39;
  fin-cumipmt-overflow-and-forms A10; fin-cumipmt-period-bound A18). The 8 new-coverage failures are the new cases' cells
  past a leading exponent of 0.3: type1 C7:C9 and G7:G9, fresh I10 and K10. They are an ulp off with the geometric sum
  and to the bit with the core, and T-4059 owns them. With the core ahead of the geometric sum at any leading exponent
  (round 1), those 8 pass and gen-m9 67a4c03afd63f988 A208, b73c6cc96e093f03 A111 and d0854366830dcdee A98 are fixed too,
  with no regression; that waits for T-4059's evidence. fin-cumprinc-underflow-ends F (nper 1000, `v^nper` not
  underflowed, within an ulp) stays: out of scope here.
- Unit test: `an_underflowed_principal_is_one_core_for_both_types` (test_fn_cumulative_lift.rs).

## Landing notes
- PR #3105 (draft), its round-2 head, task blocked on T-4059. CI on that head had not been read when the
  session's window closed. Round 1 verdicts (its first head): design and security approved, Excel fidelity requested
  changes (R4.1 split, R4.2 sweep, R2 envelope, domain, FITTED range). Round 2 has not been reviewed.
- Next: once T-4059 lands, fill and score `r4-T-3141` against this frozen kernel, record the envelope count and every
  point outside it in the Outcome, run the full panel from round 1, and enqueue. If the sweep shows the core is Excel's
  past a leading exponent of 0.3, move `underflowed_principal`'s cut (round 1 had it at any exponent, which fixed 8
  more probe cells and 3 gen-m9 cells, with no regression).
- Do not repeat: using fin-cumprinc-underflow-fresh as the R4.2 sweep (its I5 and J5 fed the expm1 cut), or a manual
  case split as the R4.1 hold-out (the fit lab's `is_holdout` split is in the Outcome).
