---
id: T-4308
title: "Read Excel's large-shape gamma window exponent on the mode chain (after T-4296)"
milestone: M9
lane: research
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4296]
campaign: visible
---

## Goal
Excel's large-shape (a >= 20) gamma density inside the window (|x - a| <= 0.2 (x + a)) is reproduced away from the
mode. Otherwise the probes that rule out each exponent model are recorded.

## Context
T-4296 (docs/excel-semantics/statistics.md, the T-4296 paragraph under "Incomplete gamma: accurate kernel"; scorer
docs/research/t4296/score.py) found the rest of the window chain. The density is exp(-c(a) - B) / sqrt(2 pi a) times
(a / x): one exponential of the summed exponent, the exact Stirling error c, and the shape over the raw argument
last. It is Excel's bit for bit at x = a (B = 0) at 120 cells of probes/t4296-gamma-density-at-mode and at 40 of
the fresh sweep, and the engine now takes it (density.rs `mode_density`). What is still unknown is B, the exponent
a ln(a/y) + y - a:
- With the chain fixed, Excel's B minus the true B, per unit of shape, is within about +-0.5 ulp(ln u) of 0 at
  shapes 256 to 4096 (T-4290). It is not on the double grid of B/a: it is many ulps of B/a off near the mode. So
  the rounding enters at the size of a ln u.
- Rejected, with the T-4296 chain (rms of the per-unit difference in ulp(ln u), fit columns A:C of
  probes/t4290-gamma-window-large-pow2, then hold-out D; chance is 0.29):
  - Loader's bd0(a, y) series 0.31, 0.31;
  - -a ln(x/a) + (x - a) 0.39, 0.41;
  - -a log1p((x - a)/a) + (x - a) 0.41, 0.39;
  - a log1p((a - x)/x) + (x - a) 0.55, 0.54;
  - a ln(a/x) + (x - a), with a/x rounded, 3.4, 4.0;
  - a (ln a - ln x) + (x - a) 35.7, 19.6;
  - x phi(a/x) 0.57, 0.59.
  Folding 1/u into the exponent as (a - 1) ln u is 32 to 500 ulp off at shapes 256 to 4096.
- Seven cells that matched on the old chain now miss by 1 or 2 ulp (decision 0286): t4255 C29, C31 and t4290 C90,
  C123, D92, D93, D95. They come back with the exponent.

Visibly-off cells this task owns (taken over from T-4296, plus the fresh sweep):
- probes/t4290-gamma-window-large-pow2 A1:F226.
- probes/t4255-gamma-window-exact-ratio A5:D58 and G15.
- probes/t3252-gamma-prefactor-holdout-2 A5:A12, A17:A24 and A29:A36; probes/t3252-gamma-prefactor-holdout A5:A14,
  A19:A28 and A33:A42; probes/t3252-gamma-prefactor-window A4:A10, A11:A20, A24:A30 and A31:A40.
- probes/t3252-gamma-prefactor-sweep A9, A19, A20, A23, A53, A54, A55, A57, A58 and A63.
- probes-numeric/t4296-gamma-density-r4-fresh A41:A160.

From the T-4296 review (PR #3315; MINOR, carried here):
- State the R4.1 margin when the hold-out split is fixed, before scoring. T-4296 stated none.
- Commit the Gate A envelope check and the per-engine fresh-sweep counts as a script under docs/research/t4308/.
  T-4296's were computed ad hoc: 160/160 inside the envelope, and 78/160 bit-exact on
  probes-numeric/t4296-gamma-density-r4-fresh.
- Probe the near-mode region T-4296 did not cover: x/a from 0.6 to 0.667 at shapes outside 30 to 500, and shapes
  above 1E7. `mode_density` takes the chain there, unprobed. Also probe a shape past the 2 pi a overflow, where
  the density keeps the old chain.
- Once the exponent moves, make the near-mode test once (density.rs `density_with` and `gamma_prefactor` both
  evaluate `excel_away_from_mode` today).

## Acceptance criteria
- [x] A candidate exponent on the T-4296 chain is scored with a hold-out fixed before scoring (decision 0188 R4),
      and adopted if it is Excel's on the hold-out. Otherwise each rejected family goes into statistics.md with its
      scores.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below 20 (T-3219).
- The cdf inside the window (it decouples from the density from about 0.72a; T-4255).

## Pre-registration (written before any candidate was scored, decision 0188 R4.1)
- Target: Excel's exponent read out as a double. For each window cell (0.6668 < x/a < 1.4999, |x/a - 1| >= 0.02)
  of probes/t4290-gamma-window-large-pow2, the double B within 128 ulp of the true exponent whose T-4296 chain
  exp(-c(a) - B) / sqrt(2 pi a) * (a / x) is Excel's value bit for bit (cells with no such B are reported and left out).
- Split: fit = columns A:C (shapes 4096, 1024, 256), hold-out = column D (shape 2048, x/a on the k + 2 grid).
- Score: the count of cells where the candidate's B is Excel's B exactly, then rms in ulp(B).
- Margin: a candidate is adopted only if its hold-out exact rate is within 10 percentage points of its fit rate and
  beats the correctly rounded B's hold-out exact rate by at least 10 points.

## Outcome
Found Excel's window exponent and adopted it. Excel's large-shape GAMMA.DIST density is one chain, Loader's dgamma:
exp(-c(a) - B) / sqrt(2 pi a) * (a / x), with B taken from bd0 cut at |y - a| <= 0.2 (y + a).
- **Read-out.** On the T-4296 chain one double B reproduces Excel's density bit for bit at 746 of 809 t4290
  window cells, so the exponent is read exactly (docs/research/t4308/score.py part 1).
- **Window.** B = (y - a) v - (2a atanh(v) - 2a v), v = fl((y - a)/(y + a)), with atanh correctly rounded (a
  double-double ln). Scored against Excel's exact B with the pre-registered split: fit (t4290 A:C) 551/551,
  hold-out (D) 193/195. The correctly rounded B is at 74/551 and 26/195. The R4.1 margin holds: the hold-out rate
  of 99.0% is within 10 points of the fit's 100% and 85.7 points above the correctly rounded B's 13.3%.
  This is a structural model, an operation order chosen from ranked candidates, with no fitted constant, so it
  has no `FITTED` tag.
- **Outside the window.** The same chain takes Loader's direct deviance a ln(a/y) + (y - a) from a/200 up. This is
  T-4319's change, folded in here because it is one chain. It is bit-exact at 156/160 probed cells above the
  window and 119/155 below it, against 43 and 65 for the old ratio prefactor (score.py part 4).
- **Window ends.** |y - a| <= 0.2 (y + a), tested in doubles, puts every t4290 E/F ladder cell and every t4255 edge
  cell on Excel's side.
- **Code.** density.rs `density_with`, `in_window`, `loader_density` (it replaces `mode_density`),
  `window_exponent`, `direct_deviance` and `atanh_rounded`. The test for the near mode is now made once for the
  density (`in_window`), and P and Q keep `excel_away_from_mode`, since the window cdf decouples (T-4255).
- **R4.2 fresh sweep.** probes-numeric/t4308-gamma-density-r4-fresh: 154 cells drawn after the freeze (seed
  43081, docs/research/t4308/make_sweep.py), excel-graph 2026-11-05. They cover the window at shapes 20 to 1E9,
  outside it, 0.6a to 0.667a at shapes outside 30 to 500, shapes 1E7 to 1E12, and the 2 pi a overflow. Of the
  54 cells with a nonzero Excel value, the engine is bit-exact at 52, within 4 ulp at 53 and within 15 digits at
  53. The correctly rounded density is at 3, 9 and 12. Gate A's envelope holds every point (score.py part 3,
  committed as the MINOR item asked). On the t4296 fresh sweep the engine is 125/126 bit-exact.
- **New quirk.** Past the 2 pi a overflow Excel's density at the mode is #DIV/0! (A152:A154). It goes to T-4340.
- **Eval delta.** probes fixed 1,168, gen-m9 fixed 1. That includes the seven cells decision 0286 let regress,
  which come back. One regression, probes/t4290-gamma-window-large-pow2 B223 at x/a = 1.5078125, where Excel
  takes the ratio prefactor and its neighbours take the chain, is recorded by decision 0290 (follow-up T-4339).
  New: probes-numeric t4308 fresh, 149 of 154 cells pass.
- **Follow-ups.** T-4339 (row 223 above the window) and T-4340 (#DIV/0! past the overflow).
