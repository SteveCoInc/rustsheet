---
id: T-4242
title: "NEGBINOM.DIST cdf at number_s 1 and a tiny p is 9.4E-8 relative off (Gate A r4-T-3576 A125)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`NEGBINOM.DIST(f, 1, p, TRUE)` at a tiny p is within Gate A's envelope, so
accuracy/NEGBINOM.DIST/r4-T-3576!Calc!A125 passes and its known-failures line goes.

## Context
- Found by T-3576's fresh Gate A batch (eval/accuracy/tables/NEGBINOM.DIST/r4-T-3576, log-uniform, excel-graph
  2026-11-04): `NEGBINOM.DIST(70991230,1,8.02387331627331E-281,TRUE)` is Excel's 5.696246441102946E-273, equal to
  the 50-digit math; the engine gives 5.696245902984186E-273, 9.4E-8 relative off.
- At number_s 1 the cdf is the incomplete beta's closed form I_p(1, f + 1) = 1 - (1 - p)^(f + 1), which NEGBINOM.DIST
  takes first (`binom.rs` `negbinom_dist_of`, `incomplete_beta::closed_form_tails`). At p = 8E-281, y = fl(1 - p) is
  1, so a form that goes through y loses p; Excel keeps the math's value here, so it works from p (for example
  -expm1((f + 1)·log1p(-p))). Check which form the closed form takes at a = 1 and whether it rounds p away.
- Not T-3576's rule: the L140 overflow check runs after the closed forms and never fires at number_s 1.

## Acceptance criteria
- [x] A125 passes Gate A; its line in eval/accuracy/known-failures.tsv is removed.
- [x] No closed-form cell that matches Excel today moves (eval-check: 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- NEGBINOM.DIST at a huge number_s (T-3576, T-4238).

## Outcome
- NEGBINOM.DIST's cumulative path (`binom.rs` `negbinom_dist_of`) now tries `incomplete_beta::shape_one_tails` after
  the closed forms. This is bratio's evidenced a0 = 1 branch for y0 > 0.9, w = 1 - e^(b·log1p(-x)) with Excel's log1p
  (statistics.md "Incomplete beta at a shape of 1 outside the closed forms", T-3007/T-3764), which BETA.DIST's kernel
  already takes. At p = 8E-281 it keeps p where fl(1 - p) is 1.
- `shape_one_tails` became `pub(crate)`. No new form and no constants: the branch is reused as it was.
- Unit test `negbinom_cdf_at_one_success_keeps_a_tiny_p`.
- Eval delta: Gate A loses accuracy/NEGBINOM.DIST/r4-T-3576 A125 (`eval-gates`: 0 failing). eval-check: 0
  regressions and 0 fixed in every suite, so no existing NEGBINOM.DIST cell moves.
