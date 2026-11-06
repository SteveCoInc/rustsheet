---
id: T-2809
title: "BINOM.INV's count at a huge trial count (Excel 1073741823 at 2^31 - 2 trials)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BINOM.INV and CRITBINOM give Excel's count at a trial count near the 2^31 - 1 limit.

## Goal note
`BINOM.INV(2147483646,0.5,0.5)` is 1073741823 in Excel and 1073744668 here, 2845 counts out; the
same at 2147483646.9 (the trials truncate first).

## Context
Found by T-2435's probe while settling the trial limit itself (docs/excel-semantics/statistics.md
"BINOM.INV and CRITBINOM take BINOM.DIST's trial limit (T-2435)"). The search is a bisection on
statrs' `Binomial::cdf` (functions/statistical/binom.rs `binom_inv_of`), which at n = 2^31 is far
enough off at the median to move the crossing; BINOM.DIST's own cdf there is the double-double walk
of `discrete.rs`, whose budget stops well below such an n (`discrete.rs` comment: Excel's limit is
between 2E9 and 1E10, T-2429). Decide which cdf the search should use at a huge n before changing
the search.
Cells: probes/t2435-dist-extremes A15 and A18 (known failures).

## Acceptance criteria
- [x] probes/t2435-dist-extremes A15 and A18 pass, with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- The trial limit itself (T-2435, done).

## Outcome
Already fixed by T-3444 (#2752), which this task predates: `binom_inv_of` bisects on `binom_cdf_or_statrs`, i.e.
BINOM.DIST's own cdf `excel_binom_cdf` (Excel's closed forms, then the bratio port of TOMS 708 on
`I_p(k + 1, n - k)`: basym near the median, bfrac away from it), and statrs only where neither that nor the
double-double walk has a branch. At n = 2^31 - 2, p = 0.5 that is DiDonato and Morris' asymptotic expansion, an
accurate cdf with no constant fitted to any cell, and it crosses alpha = 0.5 at exactly n/2 = 1073741823, Excel's
value. T-3444's baseline update already dropped A15 and A18 from the known failures (t3444-binom-range-inv-large-n
Inv A115:B115 is the same point, BINOM.INV and CRITBINOM). The search stays bounded: at most 31 bisection steps of one
cdf each under `TRIALS_LIMIT`.
- This PR: a unit test pinning the three evidenced cells (`test_binom_inv_at_the_largest_trial_count_is_excel_s_median`),
  statistics.md "BINOM.INV and CRITBINOM take BINOM.DIST's trial limit (T-2435)" no longer calls the count ours to fix,
  and `binom.rs`'s module comment no longer says BINOM.INV's search runs on statrs throughout.
- Eval delta: none (A15, A18 passed before this branch); no code path changed.
