---
id: T-4334
title: "Read the large-shape gamma window exponent from its a 2^-59 grid (after T-4308)"
milestone: M9
lane: research
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4308]
campaign: visible
---

## Goal
Excel's large-shape (a >= 20) gamma density inside the window (|x - a| <= 0.2 (x + a)) is reproduced away from the
mode, using the grid T-4308 found. Otherwise the probes and families that rule out each model are recorded.

## Context
T-4308 (docs/excel-semantics/statistics.md, the T-4308 paragraph under "Incomplete gamma: accurate kernel"; scorer
docs/research/t4308/score.py) found that Excel's implied exponent B, read on the T-4296 chain, is not one double:
where B >= a/128 it sits on the fixed grid a 2^-59 (2^-47 at shape 4096), finer than ulp(B), up to one constant per
shape; below a/128 it is on its own double grid. So the last rounding of the exponent happens at a magnitude of about
a/128. A model of B must reproduce that grid first, which rejects most forms without scoring: every form T-4255,
T-4290, T-4296 and T-4308 tried (Loader's bd0, rlog and rlog1, log1p, mantissa-and-exponent logs, fixed-grid logs)
scores at chance. Leads not tried:
- an exp split into two factors whose arguments are rounded separately (for example exp(-(B - K)) exp(-K) with K a
  multiple of a/128, or exp(-c) and exp(-B) with B split at a power of two);
- B accumulated over terms of a series in v = (x - a)/(x + a) scaled by a before the sum, where the largest
  rounded term is about a/128;
- whether the per-shape offset (column A +0.01, B -0.37, D +0.33 of the grid) is c(a)'s own rounding.

Visibly-off cells this task owns (taken over from T-4308):
- probes/t4290-gamma-window-large-pow2 A1:F226.
- probes/t4255-gamma-window-exact-ratio A5:D58 and G15.
- probes/t3252-gamma-prefactor-holdout-2 A5:A12, A17:A24 and A29:A36; probes/t3252-gamma-prefactor-holdout A5:A14,
  A19:A28 and A33:A42; probes/t3252-gamma-prefactor-window A4:A10, A11:A20, A24:A30 and A31:A40.
- probes/t3252-gamma-prefactor-sweep A9, A19, A20, A23, A53, A54, A55, A57, A58 and A63.
- probes-numeric/t4296-gamma-density-r4-fresh A41:A160.

From the T-4296 review (PR #3315; MINOR, carried over by T-4308):
- State the R4.1 margin when the hold-out split is fixed, before scoring.
- Probe the near-mode region T-4296 did not cover: x/a from 0.6 to 0.667 at shapes outside 30 to 500, and shapes
  above 1E7. `mode_density` takes the chain there, unprobed. Also probe a shape past the 2 pi a overflow, where
  the density keeps the old chain.
- Once the exponent moves, make the near-mode test once (density.rs `density_with` and `gamma_prefactor` both
  evaluate `excel_away_from_mode` today).

From the T-4308 review (PR #3349; MINOR, excel-fidelity, carried here):
- docs/research/t4308/score.py part 1 takes every cell with B > 8, which mixes in cells below a/128 that sit on their
  own double grid. Split it at a/128: restricted to B >= a/128 the spread is 0.013 g at shape 4096 (noise 0.016),
  0.053 at 1024 (noise 0.062) and 0.028 at 2048 (noise 0.031), and quote those in statistics.md.
- The cross-shape claim in statistics.md ("the same function of u at shapes 1024 and 4096 to about 3 units of
  2^-59") is rms 2.5 and max 5.5 units over 213 common u, and score.py does not print it. Add that comparison to
  score.py and correct the wording. The largest gaps sit at both ends of u (0.67 to 0.68 and 1.40 to 1.41), which may
  be a trend worth reading.

## Acceptance criteria
- [x] A candidate exponent that reproduces the a 2^-59 grid is scored on the T-4296 chain with a hold-out fixed before
      scoring (decision 0188 R4), and adopted if it is Excel's on the hold-out. Otherwise each rejected family goes
      into statistics.md with its scores.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below 20 (T-3219).
- The cdf inside the window (T-4255, T-4326).

## Outcome
Done by T-4308's second PR (#3352), which this task's PR #3349 ran in parallel with: Excel's window exponent is
Loader's bd0 split B = (y - a) v - (2a atanh(v) - 2a v), with atanh correctly rounded, one double that gives Excel's
density bit for bit (fit 551/551, hold-out 193/195; docs/research/t4308/read_out.py). The a 2^-59 grid reading is
superseded (statistics.md, the T-4308 paragraph). The near-mode and overflow probes are in
probes-numeric/t4308-gamma-density-r4-fresh, and the single near-mode test is density.rs `in_window`. The
visibly-off window cells pass, except those named in T-4339 and T-4340.
