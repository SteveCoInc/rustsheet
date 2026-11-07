---
id: T-3740
title: "BINOM.DIST.RANGE past the mean: Excel's exact upper-tail form (21 probe rows within 6E-14)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BINOM.DIST.RANGE ranges whose lower end is past the mean are bit-exact with Excel (or within its 1-ulp noise) on the
21 probe rows where Excel's range is not its own BINOM.DIST cdf difference.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Found by T-3444 (2026-11-01). probes/t3444-binom-range-inv-large-n Range rows 6, 12, 18, ..., 126 (every sixth row,
  ranges from mean + 3 sd to mean + 5 sd, n 1E4 to 2^31 - 2) differ from Excel's own
  `BINOM.DIST(s2,...)-BINOM.DIST(s1-1,...)` in column B by 2E-16 to 6E-14. statistics.md "BINOM.DIST.RANGE and
  BINOM.INV at a large n sit on BINOM.DIST's own cdf (T-3444)".
- T-3444 tried the upper-tail difference P(X > s1 - 1) - P(X > s2) (each tail bratio's w of I_p(k+1, n-k), from
  `closed_or_ported_tails`) past the double-double walk, guarded by s1 - 1 > n·p: 18 of the 21 rows landed within 1E-15
  relative, none bit-exact (A6 and A12 at n = 1E4 stayed 1.6E-14 and 2.4E-14 off, A54 9E-15; A18 and A36 are walked,
  1E-16 and 4.6E-15 off). The fidelity review dropped it: scored only on these rows, with no R4 hold-out or fresh sweep,
  and moving no cell to Pass; several rows are 2E-16 off, inside the 4-ulp web-vs-desktop floor (decision 0188 R4.4).
- Cells (probes/t3444-binom-range-inv-large-n, Range!): A6, A12, A18, A24, A30, A36, A42, A48, A54, A60, A66, A72, A78, A84, A90, A96, A102, A108, A114, A120, A126
- The probe has no range starting between the mean and mean + 3 sd: probe that band before choosing a guard (PR #2752
  reviews).
- Candidates to score: the tails' other bratio branch (w1 complement), a pmf sum over the range from the mode side, a
  threshold other than the mean (the mode, cdf(s1 - 1) > 0.5), and the walk's behavior at n = 1E4 to 1E5. Score them
  against the 21 rows plus the 105 cdf-difference rows (none may move off). A choice among ranked forms owes the
  decision 0188 R4 hold-out and fresh sweep (work-loop step 4, "Numeric fits").
- T-3741 (2026-11-04): Gate A table accuracy/BINOM.DIST.RANGE/s1 has 9 ranges above the mean carried by
  `eval/accuracy/known-failures.tsv` lines owned by this task (A85, A91, A94, A101, A107, A146, A172, A177, A180). Ours
  cancels to 0 on four of them (down to 1.9E-134) where Excel is within 7E-12 of the math; a fix must remove those
  lines (a passing listed point fails the check as stale).

## Acceptance criteria
- [x] The 21 rows' Range!A cells pass, or the remaining misses are characterized in statistics.md with the forms tried.
- [x] No other cell of probes/t3444-binom-range-inv-large-n or any gated case regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST's own last digits (T-3447), which the other failing Range and Inv cells of the probe carry.

## Outcome
- Past the walk's budget, BINOM.DIST.RANGE with its lower end past the mean (`s1 - 1 > n·p`) now returns the upper-tail
  difference `P(X > s1 - 1) - P(X > s2)` (`binom.rs` `excel_binom_upper_tail`). Each tail is the `w` of the same
  bratio call whose `w1` is BINOM.DIST's cdf. This is the cancellation-free form, not a fit, and it has no constants.
- The form was frozen, then tested on a fresh sweep drawn after the freeze: probes/t3740-binom-range-past-mean-sweep,
  seed 374020261107. Predictions were committed before the excel-graph fill (f6bd2017d, docs/research/t3740).
  - P1 held: Excel is within 1E-12 relative on 60 of 60 rows past the mean.
  - P3 held: the 10 control rows below the mean are unchanged and equal Excel's cdf difference to the bit.
  - P4 held: Excel's range is not its own cdf difference on 37 of the 60 rows.
  - P2 failed on 9 of 60 rows near the mean, where the cdf difference sits nearer Excel's bits.
  - Overall the form is visibly off on 19 rows against the cdf difference's 38, and bit-exact on 7 against 9.
  - The form was not changed after the fill.
- The 21 t3444 rows are characterized in statistics.md: none is bit-exact, and Excel's exact form is unknown. Of
  probes/t3444-binom-range-inv-large-n's Range cells, 19 change value; the earlier "no cell changed" read only
  eval-check's pass counts.
  - 17 of the 21 rows past mean + 3 sd move from visibly off to within 15 digits, for example A60 405 to 2 ulp and A48
    215 to 5 ulp. A12 (318 to 156 ulp) and A54 (471 to 66) stay visibly off.
  - A6 (`BINOM.DIST.RANGE(10000,0.3,3137,3229)`) gets worse, from 2 ulp to 113 ulp, and is now visibly off. It was
    already a bit-level known failure, so no gate moves.
  - The case's visibly-off Range cells drop from 67 to 52.
  - The form is not given a special case for A6 (decisions 0056, 0082). T-4746 owns A6 with the other rows.
- R4 guards (check 7, kind (c): an operation order with no constants, so no `FITTED` tag).
  - R4.1 hold-out. Train is the 21 t3444 rows the form was first scored on (T-3444). Hold-out is sweep rows 1-60,
    drawn after the freeze, as in T-4322 where the fresh sweep is the hold-out. No margin was set in advance.
    Counts are bit-exact / within 4 ulp / within 15 digits:

    | form | train (21) | hold-out (60) |
    |---|---|---|
    | upper-tail difference (chosen) | 0 / 8 / 17 | 7 / 33 / 41 |
    | cdf difference (before) | 0 / 2 / 2 | 9 / 20 / 22 |
    | correctly rounded math | 0 / 6 / 8 | 0 / 0 / 0 |

    The chosen form leads both alternatives at 4 ulp and at 15 digits on both sets. Its exact rate rises from train
    to hold-out (0/21 to 7/60). The fit lab's margin is one-sided (the largest allowed drop from train to test), so
    the guard holds for any non-negative margin, including the default 5 points. The split is manual, not the fit
    lab's hash split (T-3360): the form has no tunable parameter, and the hold-out was drawn after the freeze.
  - R4.2 sweep. Batch accuracy/BINOM.DIST.RANGE/r4-T-3740 has 200 points, 100% log-uniform, generated after the
    freeze and filled from excel-graph. `rustsheet-eval accuracy` reports 0 failing, but none of its 200 points
    reaches the changed branch: p log-uniform down to 1E-300 makes n·p about 0, and 197 of the points have s1 = 0.
    So, as in T-4322, the in-region evidence is the post-freeze sweep probes/t3740-binom-range-past-mean-sweep: all 60
    rows past the mean are inside the Gate A envelope, the worst 6.2E-12 relative to the math.
  - The guard `s1 - 1 > n·p` was fixed before the sweep. The evidence places Excel's own switch somewhere between
    mean - 0.05 sd (the controls are Excel's cdf difference to the bit) and mean + 0.1 sd. 23 of the 60 rows past the
    mean still have Excel's range equal to its cdf difference, so the mean is the cancellation boundary, not Excel's
    proven switch (T-4746).
- Unit test `binom_dist_range_past_the_mean_takes_the_upper_tails` (Gate A A107, A85).
- Eval delta: Gate A loses 9 known failures (accuracy/BINOM.DIST.RANGE/s1 A85, A91, A94, A101, A107, A146, A172,
  A177, A180) and gains the 200-point r4-T-3740 batch (all inside the envelope, none on the changed branch); `eval-gates` has 0 failing. The new probe case adds 85 known failures, all owned by T-4746.
