---
id: T-3879
title: "bpser: EXP of a non-double z as fl(fl(EXP(zh)·s)·fl(1 + zl)) at min(a, b) < 1, b0 < 8"
milestone: M9
lane: functions-stat
status: done
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
- [x] bpser at min(a, b) < 1 < b0 < 8 and b0 <= 1 uses the split form; unit tests pin three non-double-z cells from
      t3769-r6-bpser-beta (one positive zl, one negative, one q = 0.75) and one double-z cell.
- [x] R4.2 fresh sweep (decision 0188 R4; T-3873 chose the split on r6's tune shapes, and every set it scored was
      filled before the freeze): generate a log-uniform BETA.DIST probe over min(a, b) < 1, b0 < 8 (direct bpser
      cells, with Excel's GAMMALN at each shape's arguments), fill it from excel-graph, and record before merge its
      exact / 4 ulp / 15 digits for the split and for CR, and its Gate A envelope result.
- [x] Eval known failures only shrink; `make eval-baseline` committed; the eval delta is in the Outcome.
- [x] statistics.md's "Incomplete beta bpser" `Implemented in` line says the split is implemented.
- [x] `make check` green; reviewers approve.

## Out of scope
- b0 >= 8 L (T-3569); min(a, b) >= 1; GAMMALN accuracy (T-3042).

## Handoff (resumed)
Done on this branch: the split-z form in `bpser.rs` (`bpser_series(.., split)`; `split` = min(a, b) < 1 and b0 < 8,
F.TEST's `bpser_with_ln_beta` keeps `split = false`). `make eval` + `make eval-baseline`: 0 regressions; fixed 71 cells
in gen-m9 and 10 in ironcalc-calc (baseline files committed; the eval run's other suites unchanged). `cargo clippy` clean.
Not done (needs an opus run: numeric-fit guards of decision 0188 R4): the three non-double-z unit tests + one
double-z test from t3769-r6-bpser-beta; the R4.2 fresh sweep through excel-graph (exact / 4 ulp / 15 digits for split
and CR, Gate A envelope result) recorded in the Outcome; the statistics.md "Incomplete beta bpser" `Implemented in`
line; `make check-fast`, the review panel (design, security, excel-fidelity), then `status: done` and enqueue.

## Outcome
- `bratio/bpser.rs`: `bpser` forms I as fl(fl(EXP(zh)·s)·fl(1 + zl)) (zh = z to nearest, zl = z - zh) where
  min(a, b) < 1 and b0 < 8 (`bpser_series(.., split)`); elsewhere, and F.TEST's `bpser_with_ln_beta`, keep
  fl(EXP(z)·s) on the extended z. A z that is a double gives zl = 0 and today's I. Every bpser caller (BETA.DIST, F.DIST,
  BINOM.DIST and NEGBINOM.DIST cdfs) takes it.
- Unit test `bratio::tests::bpser_splits_a_non_double_z`: t3769-r6-bpser-beta A15 (zl = +2^-53), A2 (a tie rounded
  up, zl = -2^-53), A642 (q = 0.75) and A400 (double z), bit-exact; it fails on main's form (1 ulp on the three).
- Kind (c) structural model (reviewer-excel-fidelity check 7): no fitted constant, no `FITTED` tag. R4.1 hold-out is
  T-3873's (r6 b0 = 4.75: split 1,536 / 1,536 / 1,536 of 1,536 exact / 4 ulp / 15 digits, CR 178 / 1,353 / 1,536).
- **R4.2 fresh sweep**, drawn after T-3873's freeze (seed 3879): probes-numeric `t3879-bpser-split-fresh`, 2,301
  BETA.DIST cells log-uniform over min(a, b) < 1, b0 < 8 (x in [1e-4, 0.999], both orientations; kept only where the
  GL53 kernel's one series call is bpser on the cell's own (a, b, x)) and Excel's GAMMALN at the 864 arguments of their
  L; excel-graph 2026-11-03 (`docs/research/t3879/gen_fresh.py`, `score_fresh.py` → `score_fresh.out`). The form was
  not changed after scoring. Exact / 4 ulp / 15 digits:

  | sub-branch | z | cells | split | unsplit GL53 (z64) | CR (mpmath 50 digits) |
  |---|---|---|---|---|---|
  | b0 <= 1 | double | 371 | 371 / 371 / 371 | 371 / 371 / 371 | 92 / 365 / 368 |
  | b0 <= 1 | not | 1,181 | 1,181 / 1,181 / 1,181 | 685 / 1,181 / 1,181 | 245 / 1,132 / 1,167 |
  | 1 < b0 < 8 | double | 200 | 200 / 200 / 200 | 200 / 200 / 200 | 26 / 156 / 181 |
  | 1 < b0 < 8 | not | 549 | 548 / 549 / 549 | 310 / 549 / 549 | 83 / 405 / 477 |
  | all | | 2,301 | 2,300 / 2,301 / 2,301 | 1,566 / 2,301 / 2,301 | 446 / 2,058 / 2,193 |

  A second excel-graph fill of the case matched the first on all 3,165 cells (decision 0112).
  The engine itself (its own GAMMALN) is exact on 2,124 of the 2,301 and on 722 of the 864 GAMMALN cells
  (`engine_fresh.out`): the remaining misses are GAMMALN's (T-3042), out of scope.
- **Gate A envelope:** the split model is inside on 2,301 of 2,301 fresh-sweep cells (none outside to list). The Gate A
  batch `eval/accuracy/tables/BETA.DIST/r4-T-3879` (194 points, default strata, drawn and filled after the freeze; a
  first log-uniform-only draw with shapes up to 1e297 could not open an excel-graph session and was discarded unfilled)
  is 194 of 194 inside: `rustsheet-eval accuracy` 11,338 points, 0 failing, 0 table defects.
- R4.4: the effect is 1-2 ulp, below the 4-ulp web-against-desktop floor; these are excel-graph bits, deterministic
  over two fills here and five in T-3873, not shown for desktop Excel (decision 0112).
- Eval delta: gen-m9 known failures -71, ironcalc-calc -10, 0 regressions; the other suites unchanged (baseline
  committed). statistics.md's bpser entry gains the fresh-sweep counts and its `Implemented in` line says the split
  is implemented.

