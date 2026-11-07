---
id: T-4242
title: "NEGBINOM.DIST cdf at number_s 1 and a tiny p is 9.4E-8 relative off (Gate A r4-T-3576 A125)"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] A125 passes Gate A; its line in eval/accuracy/known-failures.tsv is removed.
- [ ] No closed-form cell that matches Excel today moves (eval-check: 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- NEGBINOM.DIST at a huge number_s (T-3576, T-4238).
