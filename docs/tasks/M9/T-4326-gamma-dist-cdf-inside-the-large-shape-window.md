---
id: T-4326
title: "GAMMA.DIST cdf inside the large-shape window: Excel's 15th digit at GAMMA.DIST(35,50,1,TRUE) and the cut where the cdf leaves the density's exponent error"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4308, T-4325, T-4350]
campaign: visible
---

## Goal
GAMMA.DIST's cdf (and Q, through CHISQ.DIST.RT and POISSON.DIST's shared kernel) at a large shape inside the
window, from 2a/3 up to the point where Excel's cdf stops sharing the density's exponent error, takes the same
in-window chain as the density, so `GAMMA.DIST(35,50,1,TRUE)` shows Excel's 15 digits. Above that cut P and Q keep
their current path.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 062b67cf8). `make unowned-off ALL=1` lists exactly one
  visibly-off cell with no live owner, `GAMMA.DIST (deferred)`. Only the deferred T-4297 names its case, and it
  leaves the case's visibly-off cells to T-4290, which is done. T-4308 took over T-4290's density cells but puts
  "the cdf inside the window" out of scope, so nothing owns this cell.
  - probes/t3252-gamma-prefactor-window.case.json (1): Calc!A44
- The cell (`make eval-show CASE=probes/t3252-gamma-prefactor-window.case.json`):
  `GAMMA.DIST(35,50,1,TRUE)` (x/a = 0.70, inside the window |x - a| <= 0.2 (x + a)). Excel 0.009845502476408533,
  ours 0.009845502476408544, mpmath 50 digits 0.0098455024764085416. Excel is 5 ulp (-8.7E-16 relative) below the
  true value and ours 1.4 ulp above, so the 15th digit differs (...853 against ...854). Excel is the inaccurate one,
  so this is the `visible` stage: reproduce Excel's algorithm (decision 0114). Decision 0188 R3 does not apply
  (Excel is 8.7E-16 off, far below 1e-4), and decision 0267 does not either (GAMMA.DIST is not in the
  incomplete-beta family).
- The quirk is recorded: docs/excel-semantics/statistics.md, "Incomplete gamma: accurate kernel", T-4255 paragraph
  ("The cdf decouples from the density from about 0.72a ... A4 and A44 ... are -9.1E-16 and -8.7E-16 relative from
  the 50-digit values, and A44 shows it"). The density A4 at the same point carries the same per-unit error. Up to
  0.70a the cdf carries the density's error; holdout A24 and A38 (cdfs at shapes 100 and 500 near 0.72a) carry only
  -0.17 and -0.13 ulp per unit against their densities' -0.65.
- Code: P and Q take `gamma_prefactor` (crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs:
  `rcomp`'s -a rlog(x/a) near the mode, Excel's ratio form away from it), while the density takes T-4296's chain
  (`mode_density` in incomplete_gamma/density.rs). The fix is likely the density's chain, with T-4308's exponent,
  as the prefactor of P and Q from 2a/3 up to a cut c. Both c and whether the cdf adds its own rounding after the
  prefactor are read from evidence, not assumed. T-4255 named Temme's expansion as one candidate above c. That
  region is not this task's target: there the cells are within 15 digits.
- **Needs an oracle sweep.** The 55 cached in-window cdf cells between 2a/3 and 0.8a are all at shapes 30 to 500,
  where the exponent's error does not read out. So this task depends on T-4325 (research, sonnet), which draws
  density, cdf and upper-tail triples at power-of-two shapes 256 to 4096 with a shape-2048 hold-out fixed before
  scoring. It also depends on T-4308, which reads the in-window exponent. Without T-4308 the cdf can only inherit
  the density's present miss.
- T-4350 (gap triage 2026-11-05, statistics.md T-4350 bullet) found that up to 0.70a Excel's cdf is its own
  density times x/a times an accurate series. That holds at 49 of 49 cached scale-1 pairs within 6 ulp, and A44 is
  +0.4 ulp from that product. T-4350 moves P onto the density's chain up to a cut, which the cached cells place
  in (0.70a, 0.7033a]. So A44 is expected to pass there, and this task then fits the cut with T-4325's sweep and
  owns the visibly-off cdf just past it:
  - probes/t3252-gamma-prefactor-sweep.case.json (1): Calc!A54 `GAMMA.DIST(269.8924,383.73,1,TRUE)` (0.7033a):
    Excel 4.1650721231146273E-11, ours 4.1650721231146435E-11, true 4.1650721231146392E-11. Excel is 126 ulp from
    its own density times x/a times the series, so the cdf has already left the density's prefactor there.
- Cells of the same cases that other tasks own, not this one: the window densities A4:A40 (T-4308), the
  within-15-digit window cdfs A45:A60 and the sweep and hold-out cells T-4297 lists (Phase 1b), and POISSON.DIST far
  above the mean (T-4291).

## Acceptance criteria
- [ ] probes/t3252-gamma-prefactor-window Calc!A44 is within half a unit of Excel's 15th significant digit. Its
      known-failure line (`F probes/t3252-gamma-prefactor-window.case.json Calc!A44`) leaves eval/baseline/probes.txt,
      and `make eval-baseline` adds no new line in any suite.
- [ ] probes/t3252-gamma-prefactor-sweep Calc!A54 is within half a unit of Excel's 15th significant digit, and its
      known-failure line leaves eval/baseline/probes.txt.
- [ ] No cell of probes/t3252-gamma-prefactor-window, -holdout, -holdout-2, -sweep, t4255-gamma-window-exact-ratio,
      t4290-gamma-window-large-pow2, t4325-gamma-window-cdf-pairs, or the CHISQ.DIST.RT and POISSON.DIST cells of
      ironcalc-calc becomes visibly off or worse in ulps.
- [ ] The cut c, and any ranked choice of chain for P and Q, carries the R4 guards of reviewer-excel-fidelity check 7
      (decision 0188):
  - hold-out counts: train (T-4325 fit columns) and hold-out (its shape-2048 columns, plus the cached t3252
    holdout cdfs) side by side, bit-exact, within 4 ulp and within 15 digits, for the chosen candidate and the
    correctly rounded one, with the margin stated before scoring;
  - fresh sweep: a new probes/ case drawn and oracle-filled after the freeze (probes-numeric is frozen, decision
    0267 item 5), with its counts recorded;
  - Gate A envelope: the fresh sweep's points inside the envelope against mpmath values the task attaches
    (GAMMA.DIST has no Gate A table yet), with every point outside listed;
  - a `FITTED(T-4326, n_cells, range)` tag on c, which is applied only inside its range, with accurate math outside;
  - the reproducibility floor: no step targets differences of 4 ulp or less (web against desktop) or bits that vary
    between fills;
  - no keep-matched branch: no band or special case defined by previously matching cells.
- [ ] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel" records c, the chain P and Q take
      inside it, and the scores. If no candidate wins on hold-out, it records each rejected one with its scores, and
      the task blocks as time-boxed (decision 0056).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083/0114: Calc!A44 of probes/t3252-gamma-prefactor-window is visibly off (Excel 0.009845502476408533,
ours ...544), and so is sweep Calc!A54 (T-4350's triage). The target is those cells within half a unit of Excel's 15th significant digit with no other cell of the
listed cases newly visibly off. The bit-level residue of the other in-window cdfs stays with T-4297
(`defer: phase-1b`).

## Out of scope
- The in-window exponent itself (T-4308) and the in-window densities.
- The cdf above the cut, unless the same chain carries it; shapes below 20 (T-3219); POISSON.DIST far above the mean
  (T-4291).
