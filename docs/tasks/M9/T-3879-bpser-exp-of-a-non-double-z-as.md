---
id: T-3879
title: "bpser: EXP of a non-double z as fl(fl(EXP(zh)·s)·fl(1 + zl)) at min(a, b) < 1, b0 < 8"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The engine's bpser matches Excel's bit pattern where z = fl(a·LN x) - L is not a double, for min(a, b) < 1 and
b0 = max(a, b) < 8.

## Context
- T-3873 (docs/research/t2464/r7-bpser-nondouble-z/RESULTS.md) found Excel's form: zh = z rounded to the nearest
  double (ties to even), zl = z - zh (exact, a double), and I = fl(fl(EXP(zh)·s)·fl(1 + zl)), every operation a stored
  double. Where z is a double, zl = 0 and this is today's I = fl(EXP(z)·s). It is exact on 1,139 of 1,139 non-double-z
  hold-out cells (t3769-r6-bpser-beta b0 = 4.75) and on 1,920 of 1,920 cells of t3570-r5-bpser-lngamma-beta.
- Code: `crates/rustsheet-engine/src/functions/statistical/bratio/bpser.rs` `bpser_with_ln_beta`, the last line
  `extended_product(excel_exp_extended(z), s)`. Apply the split only where L is GL53's stored double (min(a, b) < 1,
  b0 < 8). The min(a, b) >= 1 path holds L extended and b0 >= 8 has an unsolved L (T-3569): keep today's form there
  (not evidenced). F.TEST's tail (`f_test_tail.rs`) passes its own L: keep it unchanged unless its cases show the gain.
- fl(1 + zl) equals Excel's EXP(zl) on every zl the split leaves, so either is fine; fl(1 + zl) needs no EXP call.
- The engine's GAMMALN is not exact at every shape argument (T-3042), so cells whose L the engine gets wrong stay off.
  Score with `make eval`: the eval cases t3769-r6-bpser-beta, t3570-r5-bpser-lngamma-beta, t3570-r5-bpser-lg,
  t2464-r2-bpser-small-shapes/-runs and t2464-w2f-xruns-1 carry these cells.

## Acceptance criteria
- [ ] bpser at min(a, b) < 1 < b0 < 8 and b0 <= 1 uses the split form; unit tests pin three non-double-z cells from
      t3769-r6-bpser-beta (one positive zl, one negative, one q = 0.75) and one double-z cell.
- [ ] R4.2 fresh sweep (decision 0188 R4; T-3873 chose the split on r6's tune shapes, and every set it scored was
      filled before the freeze): generate a log-uniform BETA.DIST probe over min(a, b) < 1, b0 < 8 (direct bpser
      cells, with Excel's GAMMALN at each shape's arguments), fill it from excel-graph, and record before merge its
      exact / 4 ulp / 15 digits for the split and for CR, and its Gate A envelope result.
- [ ] Eval known failures only shrink; `make eval-baseline` committed; the eval delta is in the Outcome.
- [ ] statistics.md's "Incomplete beta bpser" `Implemented in` line says the split is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- b0 >= 8 L (T-3569); min(a, b) >= 1; GAMMALN accuracy (T-3042).

## Handoff
Done on this branch: the split-z form in `bpser.rs` (`bpser_series(.., split)`; `split` = min(a, b) < 1 and b0 < 8,
F.TEST's `bpser_with_ln_beta` keeps `split = false`). `make eval` + `make eval-baseline`: 0 regressions; fixed 71 cells
in gen-m9 and 10 in ironcalc-calc (baseline files committed; the eval run's other suites unchanged). `cargo clippy` clean.
Not done (needs an opus run: numeric-fit guards of decision 0188 R4): the three non-double-z unit tests + one
double-z test from t3769-r6-bpser-beta; the R4.2 fresh sweep through excel-graph (exact / 4 ulp / 15 digits for split
and CR, Gate A envelope result) recorded in the Outcome; the statistics.md "Incomplete beta bpser" `Implemented in`
line; `make check-fast`, the review panel (design, security, excel-fidelity), then `status: done` and enqueue.
