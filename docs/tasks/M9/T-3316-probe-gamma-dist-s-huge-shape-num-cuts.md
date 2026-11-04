---
id: T-3316
title: "Probe GAMMA.DIST's huge-shape #NUM! cuts one cell per workbook and pin them (T-3271 brackets)"
milestone: M9
lane: functions-stat
status: done
priority: P1
campaign: semantics
size: S
tier: opus
depends_on: [T-3271]
---

## Goal
GAMMA.DIST's two huge-shape `#NUM!` cuts sit where Excel's do, not merely at the smallest `#NUM!` probed so far.

## Context
- T-3271 (PR #2466; this task depends on it) puts both cuts at the smallest `#NUM!` Excel has shown:
  `HUGE_SHAPE_DENSITY_CUT` = 6.9187E+11 for (alpha - 1)·(-LN(x/beta)), A26's product (1E+9 - 1)·691.87, and
  `HUGE_SHAPE_CDF_CUT` = 1.85E+76 for the shape (`statistical/gamma.rs` `huge_shape_num_error`). It records the cut
  positions as UNVERIFIED in docs/excel-semantics/statistics.md "GAMMA.DIST at a huge shape". The brackets are
  density 6.9E+10 (0) to 6.9187E+11 (`#NUM!`) and cdf 1E+40 (0) to 1.85E+76 (`#NUM!`)
  (probes/t2435-dist-extremes A26, A92, A100, A101).
- excel-graph refused workbooks that held several of these cells (FileCorruptTryRepair, T-2982). Probe one cell per
  workbook and bisect each bracket. Also check whether the density cut depends on (alpha - 1)·LN alone, by varying
  x and beta at a fixed product, and whether the cdf cut depends on x/beta.
- Opus: the evidence is unclear, and the probing may need a workaround for the refusal.
- **Wrong-type cells, from T-3530's fit audit (probes-numeric/fit-audit-*, excel-graph; scored on main 4e2f5f3df after T-3350;
  docs/research/fit-audit/README.md)** (fit-audit-t3271-density-cut, 50 cells): the density cut's variable is not
  (alpha - 1)·(-LN(x/beta)). Excel is 0 at A43 `GAMMA.DIST(0.5,1E+15,1,FALSE)`, where the engine is `#NUM!`; and
  Excel is `#NUM!` at A9 `GAMMA.DIST(2.17173678289098E-300,1E+9,1,FALSE)` (product 6.9E11) and A48
  `GAMMA.DIST(1E-300,1E+9,1E-6,FALSE)` (6.77E11), where the engine is 0. The scale does not count: A45
  `GAMMA.DIST(1E-200,1E+9,1E+100,FALSE)` is 0. At the probed shapes 1E8 to 1E15 every cell (with t2435 A25, A26,
  A92) fits `#NUM!` where x / shape is below a cut in (2.17E-309, 1E-308], else 0 (UNVERIFIED as a form: probe
  smaller shapes, x/shape between those values). That bracket is the subnormal range, so the cut may be "x/shape is
  not a normal double". It does not explain fit-audit-t3271-cdf-cut A41 `GAMMA.DIST(2,1.85E+76,1E+10,FALSE)` and A42
  `GAMMA.DIST(2,1E+70,1E+10,FALSE)`: `#NUM!` densities at x/shape about 1E-76 and 2E-70 (the engine agrees). So from
  a shape of 1E70 a second mechanism, likely the cdf's, applies.
- The cdf cut (`HUGE_SHAPE_CDF_CUT` 1.85E76) is refuted too: fit-audit-t3271-cdf-cut (38 of 42 cells; A5, A8, A31 and A33 have no value; filled one cell
  per workbook) has 21 wrong-type cells. Excel is `#NUM!` where the engine is 0 at A4, A6, A7, A9, A10, A11, A13, A14,
  A15, A16, A17 and A40 (`GAMMA.DIST(2,1E+60,1E+10,TRUE)` up to 1.8E76; `GAMMADIST` too). Excel is 0 where the engine
  is `#NUM!` at A28, A29, A32, A34, A35, A36, A37, A38 and A39 (beta = 1, shapes 1.85E76 to 1E200). Excel is not
  monotone in the shape: A12 (1E76) and A18 (1.84E76) are 0 between `#NUM!`s, and the refill agreed. So look for an
  intermediate of Excel's arithmetic that overflows, not for a cut.

## Acceptance criteria
- [x] Each cut is pinned (or narrowed by at least 10x, with the remaining bracket recorded) by excel-graph probe cells; statistics.md is updated; the constants move if the evidence says so; `make eval-check` shows no regressions and the visible-parity lock does not rise.
- [x] `make check` green; reviewers approve.

## Out of scope
- The gamma inversions (T-3070, T-3272).

## Outcome
- **Density cut pinned (mechanism, not a fit):** Excel's GAMMA.DIST density of a positive x is `#NUM!` where
  shape / x overflows a double (cut x = shape / DBL_MAX), whatever the scale, and even where the density is about
  1E+304. Evidence, one fill per case on excel-graph: probes-numeric/t3316-gamma-density-overflow A1:A15 (pairs
  5.5626E-300 / 5.5628E-300 at shape 1E9 and scales 1 and 1E-200, the same pairs at shapes 1000 and 1E15, and shapes
  5 and 10, which show the numerator is the shape, not shape - 1) and t3316-gamma-density-overflow-nonzero (`#NUM!`
  at 5.3E-308, shape 10, scale 2.3E-308; 3.1567E+304 at 5.6E-308). The cut is pinned to 2 parts in 1E+5 at three
  shapes. It replaces T-3271's `HUGE_SHAPE_DENSITY_CUT` (`gamma_dist_of`). It is IEEE overflow (check 7 kind (b)),
  so there are no fitted constants.
- **The huge-shape `#NUM!` (cdf, and the density at x/beta below 1) is not a cut:** from a shape between 1E55 and
  1E56 at x/beta = 2E-10 (narrowed from 1E40..1.85E76), Excel turns a 0 into `#NUM!` at about a third of the
  probed x/beta, in no order (t3316-gamma-cdf-y-sweep, 47 cells at shape 1E70; t3316-gamma-cdf-huge-shape, 20). The
  density is `#NUM!` at exactly the cdf's points (12 of 12 pairs), and only x/beta counts (A12:A14). No rule over
  the two arguments fits it without fitting cells, so the engine keeps T-3271's 1.85E76 cdf cut, and the density at
  x/beta below 1 now shares it. The mechanism is filed as T-3855 (opus), which owns those cells. A third case
  (34 cells at x/beta 2E-10, shapes 1.2E55 to 1E90) was refused twice by createSession and is not committed.
- statistics.md "GAMMA.DIST at a huge shape" rewritten: rule 1 VERIFIED, rule 2 VERIFIED at the probed cells with
  an UNVERIFIED mechanism.
- Eval delta: probes-numeric (not gated) fit-audit-t3271-density-cut 47 -> 50 of 50 (A9, A43, A48 fixed);
  fit-audit-t3271-cdf-cut 17 -> 16 of 38 (A42, `GAMMA.DIST(2,1E+70,1E+10,FALSE)`, was `#NUM!` only through the
  refuted product rule; T-3855 owns it). New cases: t3316-gamma-density-overflow 24/26,
  t3316-gamma-density-overflow-nonzero 2/6 exact (the other 4 numeric, not visibly off), t3316-gamma-cdf-huge-shape
  5/20, t3316-gamma-cdf-y-sweep 32/47 (every miss owned by T-3855). Gated suites: no change (`make eval-check`).
- Follow-up: T-3855.
