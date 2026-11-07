---
id: T-3523
title: "Re-validate IPMT's DISCOUNTED_INTEREST_FROM selector on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
IPMT's `DISCOUNTED_INTEREST_FROM` (financial/annuity.rs, 6E-16) is shown to generalize on a fresh hold-out sweep, or is replaced by the accurate (exact balance) interest.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- The cut chooses between IPMT's exact balance interest and Excel's own discount (`discounted_interest`) by their
  relative difference; tagged `FITTED(T-2587, 8 cells, IPMT nper 167..1E300, rate -0.05..10)`, financial.md "IPMT
  follows Excel's own discount where it moves the value". Eight cells is a thin fit.
- Method: build the engine twice (with the selector, and with the accurate form only) and score both on the old
  cells through the fit lab (`docs/architecture/fit-lab.md`, hold-out split, default margin 5 points), then on the
  fresh sweep. Accuracy-suite tables are refused as sweeps (`--holdout-only` measures a frozen fit only).

## Acceptance criteria
- [ ] A fresh log-uniform sweep over the selector's full domain (R4.2), new cells in probes-numeric that no earlier fit
      saw, is oracle-filled and committed.
- [ ] The Outcome reports, with the selector and with the accurate form (two engine builds, or fit-lab candidates that
      reproduce each bit for bit on the old cells), the fit lab's train and test counts (T-3360's hold-out split and
      margin) on the old cells and the counts on the fresh sweep.
- [ ] If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
      accurate form on test and on the fresh sweep, it is replaced by the accurate form (one `FITTED` tag fewer in
      quality/fitted.json via `scripts/check_fitted.py --update`); otherwise it keeps its `FITTED` tag, with the range
      restated to what was measured, and is not applied outside that range (R4.3).
- [ ] The Gate A envelope (R2: within 1e-9 relative of 50-digit mpmath or within 15 significant digits of Excel) holds
      on the fresh sweep, with mpmath values recorded in the Outcome or the accuracy tables.
- [ ] No gated cell regresses beyond decision 0083's allowance; a replaced selector's lost bit-exact cells are listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other selectors split from T-3365 (T-3502, T-3503, T-3523, T-3524, T-3525, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).

## Outcome
Replaced by the accurate form: `DISCOUNTED_INTEREST_FROM` and `discounted_interest` are gone (one `FITTED` tag fewer:
89 against the lock of 90; quality/ is a protected path, so the steward lowers the lock). Where T-2243's discounted
split declines, `period_payments` takes the exact balance interest.
- Why: since T-2243 the selector runs only where `discounted_split` declines (a non-normal payment factor at a
  subnormal or tiny rate, a negative rate whose `(1+rate)^-nper` overflows, an overflowing product). T-2587's eight
  cells now all go through the split.
- Old cells (R4.1): two engine builds, the selector at 6E-16 and turned off (`f64::INFINITY`). `make eval` on
  origin/main 6523a495 with it off: 0 regressions and 0 fixed in every gated suite, visible parity 2525 cells (lock
  3156). The train and test counts are the same for both forms on any hold-out split, because the selector decides
  no old cell, so the fit lab's split has nothing to separate.
- Fresh sweep (R4.2): probes-numeric/t3523-ipmt-selector-sweep, 800 cells from docs/research/t3523/make_sweep.py
  (seed 3523), oracle-filled from excel-graph 2026-11-05.
  - Column A: 200 negative rates from -0.01 to -0.9, with nper 1 to 8 times past the power's range.
  - Column B: 200 subnormal rates of either sign, written as kept-subnormal differences `(aE-308-bE-308)`. The first
    version wrote them as `2^-k`, which flushes to 0; review round 1 caught it.
  - Column C: 200 rates from 1E-3 to 10, with pv and fv from 1E+290 to 9.9E+307.
  - Column D: 200 rates from ±1E-300 to ±1E-12, with nper up to 1E+9.
- Reach: an origin/main build that logs the selector's branch shows 234 cells reaching it: B 154, C 5, D 75 (none in
  column A, where `payment_pair` is `#NUM!` first). On none of the 234 does the selector change the value: the
  discounted and exact interest never differ by more than 6E-16.
- Both builds give the same value on all 800 cells: 480 pass and 320 are visibly off.
  - **Excel is `#NUM!` on every one of the 234 cells that reach the branch.** No finite Excel value lies where the
    selector runs, so it can neither match nor miss one, and keeping it would be a fit with no evidence.
  - The 320 visibly off: Excel `#NUM!` against the engine's finite value on all 200 of B, 113 of D and 6 of C, plus
    C139, which is 13 digits close.
  - All of them are new findings, not the selector's, owned by T-4314 (filed in this PR, cells listed there) and
    recorded in financial.md "IPMT near the double range and at tiny rates".
- Gate A (R2): IPMT has no Gate A definition, so `rustsheet-accuracy math` writes no mpmath sidecar for this case and
  the envelope is judged against Excel's 15 digits only.
  - Where the selector runs, Excel has no number to compare: all `#NUM!`, a type mismatch T-4314 owns.
  - The 480 passing cells are within 15 digits of Excel.
  - C139 (`IPMT(0.838240020339469,5,15,1.31635124625391E+294,4.11149200495181E+298,1)`) does not reach the branch.
    Excel gives 2.052481555601429E+295 and the engine 2.052481555601641E+295. A 60-digit mpmath of the closed form
    gives 2.0524815556015393E+295, about 5E-14 from each, inside 1e-9; T-4314 owns it.
- Eval delta: none in the gated suites. probes-numeric changes one case from 600 cells to 800 cells, 480 passing; it
  has no baseline file.
- This is not a fit: a fitted constant is removed and nothing is tuned.
