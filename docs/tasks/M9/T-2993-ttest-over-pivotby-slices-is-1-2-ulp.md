---
id: T-2993
title: "TTEST over PIVOTBY slices misses Excel's last digits (probes/pivotby-ttest-array2 P!A36, A37, A42, A43)"
milestone: M9
lane: functions-stat
status: dropped
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`TTEST(Data!C10:D12,<slice of PIVOTBY(..)>,1,2)` in probes/pivotby-ttest-array2 P!A36, A37, A42 and A43 returns Excel's
digits; ours differ by 1-2 ulp in A37, A42, A43 and by about 8 ulp in A36 (Excel 0.030297981348950877, ours
0.030297981348950905), so A36 may be more than summation order.

## Context
Found by T-2854 (excel-graph probe added by its handoff, 2026-10-28). After T-2854 the PIVOTBY grid matches Excel cell for cell
(blank keys are true blanks), so array2 holds Excel's values and the difference is in T.TEST's own arithmetic over these
inputs (type 2, one tail): the summation order or variance formula. Every other TTEST cell of the case passes, so compare
the formula path for these four inputs against the passing ones (`rustsheet-eval show probes/pivotby-ttest-array2.case.json`).

## Acceptance criteria
- [ ] The four known-failure lines leave eval/baseline/probes.txt with no new ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY itself (T-2854).

## Outcome
Dropped (2026-11-05, opus): the four cells are last-digit (Phase 1b) cells, so they move to T-3254, the deferred owner
of T.TEST/TTEST cells already within 15 significant digits (decision 0188 R5). Measured on main fb3ea5fec with the engine's
own sample arrays (the slices T.TEST reads, printed by a temporary, uncommitted trace) and mpmath at 50 digits:

| Cell | df | Excel | Ours | Ours − Excel | True − Excel |
|---|---|---|---|---|---|
| P!A36 | 7 | 0.030297981348950877 | 0.030297981348950905 | +8 ulp | +4 ulp |
| P!A37 | 13 | 0.002685332094296944 | 0.0026853320942969443 | +1 ulp | −2.5 ulp |
| P!A42 | 22 | 0.3571628041352982 | 0.3571628041352981 | −2 ulp | −9.6 ulp |
| P!A43 | 41 | 0.31632865142618277 | 0.3163286514261829 | +2 ulp | +2.7 ulp |

- Not the statistic. Sum of squares two-pass, one-pass (Σx² − (Σx)²/n, (nΣx² − (Σx)²)/n), fsum and corrected two-pass,
  each with seven ways to form t² from the pooled variance (t then squared, d²/(sp(1/n1+1/n2)), d²n1n2/(sp(n1+n2)),
  d²/(sp/n1+sp/n2), ...), all give the same y = fl(df/fl(df+t²)) at A37, A42 and A43 (y is near 1 there, so t²'s last
  bits do not reach it), and at A36 every variant gives our y. The exact I_y(df/2, 1/2)/2 at that y is our value at
  A36 (8.2 ulp above Excel), A42 and A43.
- Not the y model: Excel at A42 is 2 ulp from ours, while one ulp of y moves p by about 27 ulp, so no double y gives
  Excel's p through an exact tail. The residue is Excel's incomplete-beta noise (statistics.md "T.TEST rounds y",
  "Student t prefactor per df"); at A36 (df 7) part of it is the df-7 tail error of about −5 × 2^-53 recorded there.
- All four are within half a unit of the 15th significant digit (A36: 2.8E-17 against 5E-17).
- Eval delta: none (docs only). The four lines stay in eval/baseline/probes.txt under T-3254.
