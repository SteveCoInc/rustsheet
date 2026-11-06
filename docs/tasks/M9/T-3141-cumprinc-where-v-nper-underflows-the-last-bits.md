---
id: T-3141
title: "CUMPRINC where v^nper underflows: the last bits of the balance-drop principal (T-3067's seven regressed near-1E15 cells and the underflow-ends misses)"
milestone: M9
lane: functions-financial
status: done
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
Also owned here (salvaged from #3105 by T-4385; each CUMIPMT cell amplifies the underflowed principal's last bits): probes/fin-cumprinc-underflow-fresh C1, C2, C6, D1, D2, D6, D8, G1, G2, G3, G4, G6, H1, H2, H3, H4, H8, H9, H10, O7, P7, S2, T2, T9 (e.g. G4 `CUMIPMT(0.0004,3E9,3300000,3E9-6,3E9,0)` Excel -14.766276925198326, ours -14.766276925200145).
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
Resumed after T-4059 (PR #3761), which supplied the R4.2 sweep this task was blocked on. The branch merged `origin/main`
because cumulative.rs conflicted with T-3524 and T-4349; the core was applied again on top of their forms.
- **The rule.** Where `1 - v^nper` is 1, Excel's principal over a short range is one core for both types,
  `C = pv·((1-v^m)·v^(nper-end+1))`: type 1 is `-C` and type 0 is `-(C·(1+r))`.
  - `1-v^m` is PMT's `discount_complement`: Kahan's expm1 below m·L = 1, `1 - EXP` from 1 on.
  - The power is Excel's EXP of `-(nper-end+1)·L`, and L is `growth_log`'s.
  - T-3067's form folded the `1+r` into the power, which is what left the last bits off.
- **The engine.** `underflowed_principal` takes both types, except a type 1 range from period 1. It now comes first
  of every form of a short range, at every leading exponent, not only below `GEOMETRIC_MIN_LEAD` (0.3) as T-3067's
  form did.
- **Evidence.** financial.md "Where `v^nper` underflows, the principal is one core for both types" holds the search,
  and "CUMPRINC where `v^nper` underflows, at deep leading exponents (T-4059)" holds the deep-lead evidence.
- **Structural model** (decision 0188 R4, kind (c)). It was found by enumerating every expression tree over the
  factors, with the 132 principals of fin-cumprinc-near-1e15 and fin-cumprinc-underflow-ends as the fit set.
  - New probes, excel-graph 2026-11-04: fin-cumprinc-underflow-type1 (72 cells) and fin-cumprinc-underflow-fresh
    (200 cells).
  - The fresh case's I5 and J5 (m·L 1.40) set the expm1 switch, so that case is fit data too.
- **R4.1**, the fit lab's split (`fitlab.holdout.is_holdout`, margin 5 points), over all 304 underflowed principals:

  | candidate | train exact / 4 ulp / 15 digits | test exact / 4 ulp / 15 digits |
  |---|---|---|
  | the core (kernel) | 216 / 216 / 216 of 216 | 88 / 88 / 88 of 88 |
  | correctly rounded | 65 / 216 / 216 of 216 | 33 / 88 / 88 of 88 |

- **Dropping the 0.3 cut** (the leading-exponent choice). The fit data was:
  - the probe cells type1 C7:C9 and G7:G9, and fresh I10 and K10, each an ulp off with the geometric sum and to the
    bit with the core;
  - gen-m9 67a4c03afd63f988 A208, b73c6cc96e093f03 A111 and d0854366830dcdee A98;
  - T-4059's first aimed draw, accuracy/CUMPRINC/e-T-3141 and CUMIPMT/e-T-3141, filled 2026-11-06 21:53 UTC. The
    frozen kernel is exact on 52/52 below 0.3 and on 63/148 from 0.3 up, where the geometric sum takes over. The core
    at every lead is exact on 200/200, and CUMIPMT on 199/200: A41 is T-4710's.
  The form froze at 0a1f01a (2026-11-06 22:20:15 UTC). The aimed hold-out e-T-4059 was filled after the freeze, from
  22:20:43 on. On it the form is exact on CUMPRINC 199/200 and CUMIPMT 200/200; A119, a result near 1E-285, is
  T-4710's. Main gets 81 and 151.
- **R4.2**, the fresh sweep accuracy/{CUMPRINC,CUMIPMT}/r4-T-3141, 100% log-uniform over the full domain, drawn and
  filled after that freeze (T-4059):
  - The frozen form matches main on every point except 4 CUMPRINC principals from a lead of 0.3 up. It gets all 4
    to the bit (24 of those 39 against main's 20) and loses none.
  - Only 7 of the sweep's CUMPRINC points are in the region (short, `v^nper` underflowed). The form gets all 7 and
    main 3; the aimed hold-out covers the region itself.
  - R2, the envelope: on both engines 28 CUMIPMT points fail, all of them tiny-rate `PMT·m - CUMPRINC` cancellations
    owned by T-4390 (T-4059 lists them in known-failures.tsv). Every other point is in the envelope, and so is
    every CUMPRINC point.
- **R4.3.** `FITTED(T-3141, 25 cells, CUMPRINC underflowed m*L 0.36..1.40; cut bracketed 0.9783..1.3976)` goes on
  `PAYMENT_EXPM1_TO`. quality/fitted.json for annuity.rs goes from 3 to 4 (decision 0327). There is no keep-matched
  branch.
- **Also found.** On 58 of 60 probed pairs, Excel's type 1 CUMPRINC there is its type 0 divided by `1+r` in doubles,
  recorded in financial.md.
- **Eval delta** (`make eval-baseline` on main merged in): probes fixed 214, gen-m9 fixed 3, 0 regressions, visible
  parity 6638 against an allowance of 6741.
- **Unit tests** (test_fn_cumulative_lift.rs): `an_underflowed_principal_is_one_core_for_both_types` and
  `an_underflowed_principal_keeps_the_core_past_a_leading_exponent_of_0_3`.

## Landing notes
- **State.** PR #3105 is still a draft at head 8a531e9 plus this notes commit. This session could not mark it ready
  for review: "steveash does not have permission to mark the pull request ready". CI has not run since 2026-11-04,
  because drafts do not run CI here.
- **Reviews.** Security, design and Excel fidelity all returned APPROVE on 8a531e9. Their MINORs are in the PR body and
  in T-4716. This commit adds only these notes and the T-4716 task file, so no reviewer needs to run again.
- **Order.** It must land after #3761 (T-4059). That PR adds the Gate A batches e-T-3141, e-T-4059 and r4-T-3141 and
  the financial.md entry this PR cites. At the window's close #3761 was approved, with CI running on 3a0c72c.
- **Next step.** Mark #3105 ready for review. Once #3761 has merged and CI is green here, enqueue it with SQUASH.
  - If main conflicts again in cumulative.rs or financial.md, merge `origin/main` in and run `make eval-baseline`.
  - The code change is small: `underflowed_principal`'s body, the call ahead of `short_principal` in `principal()`,
    and dropping its call from `short_principal`.
- **Do not repeat.**
  - Do not score a batch and then call it a fresh sweep. Round 1 of #3761 rejected an aimed `r4` batch, and round 2
    rejected calling post-freeze data fit data.
  - Do not cite the old aimed `r4-T-3141`: it is e-T-3141 now.
