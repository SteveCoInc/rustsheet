---
id: T-4635
title: "GAMMA.DIST's #NUM! at a finite x / beta of 1E308 at shapes 1.5 and 2: bracket it and match"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine gives Excel's `#NUM!` where GAMMA.DIST (both forms) is `#NUM!` at a huge finite x / beta, by a rule that
oracle evidence brackets, and the Gate A definition gives those points a value or a domain error by that rule.

## Context
- T-4625 found that `GAMMA.DIST(1E300,a,1E-8,..)` (x / beta = 1E308, finite) is `#NUM!` for the cdf and the density
  at shapes 1.5 and 2, and a number (1 / 0) at shapes 0.5, 1, 2.5, 2.9, 3, 10 and 1E5; at shape 2 it is a number
  from x / beta = 1E200 down. Neither "y^a overflows" (2.5 is fine) nor "a * y overflows" (1.5 fails) fits.
- Off cells (the engine gives 1 and 0): probes/t4625-gamma-dist-ratio-overflow B9, F9, B10, B11.
- statistics.md, "GAMMA.DIST's density is #NUM! at a subnormal x / beta (T-2738)" (the T-4625 paragraph); the Gate A
  definition gives no value there (tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py `HUGE_RATIO_*`).
- Engine: crates/rustsheet-engine/src/functions/statistical/gamma.rs `gamma_dist_of`.

- Probe 1 (2026-11-07, excel-graph). Predictions were committed before the fill (d17d413e8). The case and oracle are
  on branch research/t4635-probe-data at 3a5d849c6 (`eval/suites/probes/cases/
  t4635-gamma-dist-huge-ratio.*`, `docs/research/t4635`). They were not landed in eval: their 106 `#NUM!` cells would
  be new known failures that no rule found so far can pass. The cells are GAMMA.DIST(x, a, 1E-8, ..) at
  x / beta = 1E216, 1E258, 1E298, 1E306, 1E307, 1E308, 1.5E308 and 1.7E308.
  - The cdf and the density are `#NUM!` in exactly the same cells.
  - Shape 1 is never `#NUM!`.
  - 1.01 is `#NUM!` from 1E306, and 1.1 from 1E298 (not monotone in the shape).
  - 1.25 to 2.1 (1.25, 1.4, 1.5, 1.6, 1.75, 1.9, 1.99, 2, 2.01, 2.1) are `#NUM!` from 1E308.
  - 2.25 to 2.51 (2.25, 2.4, 2.45, 2.49, 2.5, 2.51) are `#NUM!` from 1.5E308.
  - Every other cell is a number (cdf 1, density 0).
  - CHISQ.DIST and CHIDIST at df 2 to 5 and x 1E200 and 1E290 are numbers. Excel for the web refuses a workbook
    holding x literals of 1E307 or 1.7E308.
  - The hypothesis "`#NUM!` exactly for 1 < a < 2.5 at 1E308" fails: 1.01 and 1.1 cut far lower, and 2.25 to 2.51
    also fail from 1.5E308. Neither y^a, a·y nor y/(a - 1) overflowing fits the cuts. A per-shape threshold table
    would be a fit, so the next step is to find the expression that overflows (Excel's incomplete gamma at a huge
    x: series against continued fraction, its prefactor, or lnΓ terms), and to bisect the 1.01 and 1.1 cuts.

## Acceptance criteria
- [x] A probe brackets the region: shapes between 1 and 2.5 (e.g. 1.1, 1.25, 1.75, 2.25, 2.4) and x / beta between
      1E200 and 1.7E308 at shape 2 and 1.5, both forms; the rule is recorded in statistics.md.
- [x] The bracket also finds the ratio threshold at shape 1.5 and at the other shapes listed (the 1E200 bound of
      `HUGE_RATIO_MIN` is evidenced only at shape 2), and covers CHISQ.DIST and CHIDIST at df 3 and 4 (shapes 1.5 and
      2 at x / 2), which may share the `#NUM!` if it comes from the shared incomplete gamma (T-4625 excel-fidelity
      review, PR #3672).
- [x] The engine matches every cell of probes/t4625-gamma-dist-ratio-overflow and the new probe (B9, F9, B10, B11
      included), and the Gate A definition's `HUGE_RATIO_*` no-value region narrows to what the rule cannot cover.
- [x] `make check` green; reviewers approve.

## Out of scope
- The overflowing x / beta rule (T-4625; the engine already matches it).
- The overflowing x / beta rule's low end: below shape 0.01 only the cdf is evidenced (GAMMADIST/s1 A182); a density
  cell there would settle it, but that is not this task's.

## Progress
- 2026-11-07: resumed by an opus run (branch idle 4 h, no PR); first fill in, second bracket probe next.

## Outcome
- Four probes (excel-graph 2026-11-07; generators, predictions committed before each fill and results in
  docs/research/t4635): t4635-gamma-dist-huge-ratio (320 cells), -cut (768), -fresh (104, drawn after the rule froze)
  and t4635-chisq-huge-x (38). Rule R (statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"): above shape 1 a
  finite y = x / beta is `#NUM!` where y / a * 4 overflows, and to shape 1.1 also where y^a does; at shape 1 and below
  never. The cut is on y (beta 1E-8, 1E-100, 1E-200 agree), both forms, and CHISQ.DIST, CHISQ.DIST.RT and CHIDIST
  share it at x / 2 (only df 3 reaches it at a finite x).
- Engine: gamma.rs `huge_ratio_num_error`, called by `gamma_dist_of` and chisq.rs's cdf/density and right tail.
  Every cell of the four probes and of probes/t4625-gamma-dist-ratio-overflow (B9, F9, B10, B11 included) passes.
- Gate A: `_huge_ratio_domain` replaces the `HUGE_RATIO_MIN` / `HUGE_RATIO_SHAPES` no-value box with rule R (GAMMA.DIST,
  CHISQ.DIST, and a new CHISQ.DIST.RT domain); no value only in the unprobed edges, shapes in (1.1, 1.11) where y^a
  overflows and shapes below 1.01 where only y / a * 4 does. Sidecars refreshed for the changed definitions.
- R4 guards (decision 0188). The 4 / a quotient is a structural model chosen from ranked candidates (4 - a,
  exp(k (4 - a)), a / (a - 1) and 4 / a scored on the second fill's brackets; only 4 / a fits all), so it owes the
  hold-out and fresh sweep: hold-out, the first fill (320 cells, fixed before the second probe was written; the model
  was chosen on the second) fits 320/320; fresh sweep, gen3 after the freeze, 104/104 (84 cut-straddling points at
  0.97 and 1.03 of the predicted cut). The answer is a domain error or 1 / 0, so the bit-exact / 4-ulp / 15-digit
  counts are all equal to the cell counts, and the correctly rounded candidate (no `#NUM!` at a finite y) scores
  0 of the `#NUM!` cells. Gate A envelope: the fresh sweep's points are domain errors or 1 / 0 by rule R, all inside.
  `HUGE_RATIO_POWER_SHAPE_MAX` = 1.1 is a cut inside its evidence bracket [1.1, 1.11), tagged FITTED (fitted lock
  96 -> 97 by `check_fitted.py --update`): its hold-out is the first fill (1.1 at 1E298 and 1.25 to 2.51 below 1E308,
  fixed before the cut was placed, all as the cut predicts) and its fresh check the Cut sheet of gen3 (shapes 1.11 to
  1.14, 16 of 16 numbers where y^a overflows).
- Eval delta: +1,230 passing probe cells in the four new cases; probes/t4625-gamma-dist-ratio-overflow 58 -> 62 of 62.
