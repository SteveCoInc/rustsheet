---
id: T-4350
title: "GAMMA.DIST cdf at a large shape up to 0.70a is Excel's own density times x/a times the series (7 visibly-off t3252 cdfs no live task names)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.DIST's cdf (and CHISQ.DIST's, through the same lower-gamma kernel) at shapes from 30 to 500, from x/a 0.60 up
to about 0.70, is Excel's own density at the same point times x/a times an accurate series. Then the seven
visibly-off cdfs below show Excel's 15 digits, and the other cdfs of the same cases come within a few ulp.

## Context
- Gap triage 2026-11-05 (`make eval` on a branch at ff265dc3, three commits behind origin/main 7f8e41ab0, none
  of which touch `functions/statistical`; excel-graph sidecars). `scripts/exit_owners.py --all-off` lists 16
  visibly-off cells with no live owner. Six of them are not this task's: one is fixed on main (the T-4014
  sheet-scoped name probe, by T-4049) and five are POISSON.DIST cells of T-4322's fresh sweep, still on its
  unmerged branch. Another two belong to T-4342, whose text wrapped their case name over a line break. This
  triage fixed the wrap. The remaining eight are all GAMMA.DIST cdfs at x/a 0.668 to 0.703. This task owns the seven at
  or below 0.70a. The eighth, sweep A54 at 0.7033a, is past the cut and goes to T-4326.
  T-4308 took the window densities of these cases (now passing) but left the cdfs out of scope. T-4297
  (`defer: phase-1b`) names the within-15-digit cells of the same cases only. T-4326 owns the window cdf
  `GAMMA.DIST(35,50,1,TRUE)` and the cut where the cdf leaves the density. It waits on the T-4325 sweep at
  power-of-two shapes 256 to 4096.
- The cells (`make eval-show CASE=probes/t3252-gamma-prefactor-holdout-2.case.json`, `...-sweep.case.json`); "true"
  is the 50-digit P(a, x):

  | cell | formula | x/a | Excel | ours | Excel vs true | ours vs true |
  |---|---|---|---|---|---|---|
  | holdout-2 A6 | `GAMMA.DIST(26.72,40,1,TRUE)` | 0.668 | 0.009726481362011727 | ...744 | -1.3E-15 | +4.4E-16 |
  | holdout-2 A18 | `GAMMA.DIST(46.76,70,1,TRUE)` | 0.668 | 0.000898008548499967 | ...9970 | -3.0E-15 | +3.4E-16 |
  | holdout-2 A30 | `GAMMA.DIST(200.4,300,1,TRUE)` | 0.668 | 3.322500627573274E-11 | ...33106E-11 | -4.5E-15 | +6.6E-15 |
  | holdout-2 A32 | `GAMMA.DIST(201.6,300,1,TRUE)` | 0.672 | 6.069835552768215E-11 | ...68172E-11 | +7.7E-15 | +6.6E-16 |
  | holdout-2 A36 | `GAMMA.DIST(204,300,1,TRUE)` | 0.680 | 1.9632442549049713E-10 | ...49576E-10 | +9.5E-15 | +2.5E-15 |
  | sweep A20 | `GAMMA.DIST(177.0467,264.4,1,TRUE)` | 0.6696 | 5.581131790163172E-10 | ...63251E-10 | -6.6E-15 | +7.6E-15 |
  | sweep A58 | `GAMMA.DIST(97.0431,140.4,1,TRUE)` | 0.6912 | 2.1218476613554785E-5 | ...54707E-5 | +2.5E-15 | -1.2E-15 |

  Excel is the inaccurate one, by up to 85 ulp, so this is the `visible` stage: reproduce Excel's algorithm
  (decision 0114). Decision 0188 R3 does not apply (Excel is under 1E-14 off) and decision 0267 does not either
  (GAMMA.DIST is not in the incomplete-beta family). At A30 and sweep A20 ours is also 7E-15 off: the 0.668a tie
  T-3252 recorded, where `gamma_prefactor` takes `rcomp` rather than the ratio form.
- The finding (docs/research/t4350/cdf_pairs.py, recorded in docs/excel-semantics/statistics.md "Incomplete gamma:
  accurate kernel", T-4350 paragraph): on every t3252 pair (density and cdf at the same x, a, scale 1) from 0.60a
  to 0.70a, Excel's cdf is within 6 ulp of Excel's own density times (x/a) times the exact series
  S = P / (f x / a). That holds at 49 of 49 pairs, at shapes 30 to 500. Against the true P those cells are up to
  267 ulp off: holdout A30 at a = 500 and 0.62a is +267.3 ulp against truth and -2.8 against the prediction, and
  holdout-2 A34 is +102.4 against -6.0. So the cdf takes the density's prefactor, with the density's whole error,
  and only the series adds rounding. Above 0.70a this breaks: sweep A54 (0.7033a, a = 383.73) +125.9, sweep A36
  (0.7063a) +23.5, holdout A38 (0.72a, a = 500) +56.7. That is the decoupling statistics.md's T-4255 paragraph
  places "from about 0.72a". The cdf `GAMMA.DIST(35,50,1,TRUE)` that T-4326 owns (window A44, 0.70a) is +0.4 ulp
  from the prediction, so it is expected to pass here too.
- Code: P and Q take `gamma_prefactor` (crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs:
  `rcomp` near the mode, the ratio form away from it, `NEAR_MODE_BELOW` 0.334 at `NEAR_MODE_BELOW_SHAPES` 30 to
  500). The density takes `density_with` (incomplete_gamma/density.rs: T-4308's `window_exponent` inside
  |x - a| <= 0.2 (x + a), `direct_deviance` outside). Expected fix: for P at a shape in the evidenced range and
  x/a up to the cut, build the prefactor from the density's own chain (density times x/a, the scale divided out
  as the density does) and multiply by the existing series. The series' own operation order is the remaining
  0 to 6 ulp, and that is Phase 1b.
- The model has no free constant inside the band. The fitted choices are the cut (somewhere in (0.70a, 0.7033a]
  on the cached cells) and the band's lower end. 0.60a is the lowest scale-1 pair cached; the T-4297 cells below
  the window, at 0.5594a to 0.6645a, may follow the same rule. Triage looked at all four t3252 cases, so none of
  them is an unseen hold-out: the fresh sweep is the hold-out.

## Acceptance criteria
- [ ] The seven cells above are within half a unit of Excel's 15th significant digit. Their known-failure lines
      (`F probes/t3252-gamma-prefactor-holdout-2.case.json Calc!A6`, `A18`, `A30`, `A32`, `A36` and
      `F probes/t3252-gamma-prefactor-sweep.case.json Calc!A20`, `A58`) leave eval/baseline/probes.txt, and
      `make eval-baseline` adds no new line in any suite (no cell of t3252-gamma-prefactor-window, -holdout,
      -holdout-2, -sweep, t4255-gamma-window-exact-ratio, t4290-gamma-window-large-pow2 or the CHISQ.DIST,
      CHISQ.DIST.RT and POISSON.DIST cells of ironcalc-calc gets worse in ulps).
- [ ] The cut and the band's lower end carry the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
  - hold-out counts: the cached t3252 cdfs (fit) and the fresh sweep (hold-out) side by side, as bit-exact,
    within 4 ulp and within 15 digits, for the chosen chain and for the correctly rounded P, with the margin
    stated before the fresh sweep is scored;
  - fresh sweep: a new probes/ case (probes-numeric is frozen, decision 0267 item 5), drawn by a committed
    generator after the cut is frozen in a commit, with density and cdf pairs at shapes 20 to 500 from 0.55a to
    0.70a and a ladder across the cut at three shapes, oracle-filled (excel-graph), with its counts recorded;
  - Gate A envelope: the fresh sweep's points inside the envelope against mpmath values the task attaches
    (GAMMA.DIST's Gate A table is T-3925's), with every point outside listed;
  - a `FITTED(T-4350, n_cells, range)` tag on the cut and on any shape bound. Each applies only inside its
    fitted range, with the current path outside;
  - the reproducibility floor: no step targets differences of 4 ulp or less (web against desktop) or bits that
    vary between fills;
  - no keep-matched branch: no band or special case defined by cells that already matched.
- [ ] docs/excel-semantics/statistics.md (the T-4350 paragraph) records the chain P takes, the cut and the
      scores. If the hold-out rejects the chain, it records the scores and the task blocks as time-boxed
      (decision 0056).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083/0114: the seven cells listed under Context are visibly off. The target is those seven within half a
unit of Excel's 15th significant digit, with no cell of the listed cases newly visibly off. The ten within-15-digit
cdfs of the same case that no other task names, probes/t3252-gamma-prefactor-holdout-2 Calc!A4, A8, A10, A12, A14,
A20, A22, A24, A28 and A34, are predicted within 6 ulp too. Their scores go in the Outcome. The bit-level residue
(the series' last ulps) of those cells and of the seven moves to a `defer: phase-1b` follow-up filed at close, or to
T-4297.

## Out of scope
- The cdf above the cut, where it leaves the density (T-4326, with T-4325's sweep), and sweep A54.
- Q far above the mean (T-4291, T-4320) and POISSON.DIST in the window (T-4322's follow-ups).
- Shapes below 20 (T-3219) and the large-shape density itself (T-4308, T-4339, T-4340).
