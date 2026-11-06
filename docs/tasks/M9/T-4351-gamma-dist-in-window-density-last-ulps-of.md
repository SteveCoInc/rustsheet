---
id: T-4351
title: "GAMMA.DIST in-window density: last ulps of the three T-4296 and T-4308 fresh-sweep misses at a non-unit scale (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The three in-window GAMMA.DIST densities that T-4296's and T-4308's fresh sweeps still miss, all within 15
significant digits, give Excel's bits, or the Outcome records why they cannot.

## Context
- Gap triage 2026-11-05 (`make eval` on a branch at ff265dc3, excel-graph sidecars). These are failing cells that
  no open task names, deferred or not: `scripts/exit_owners.py`'s ownership rule, applied to every failing cell
  of the report and not only the visibly-off ones. T-4296 and T-4308, which drew the two fresh sweeps, are done.
  T-4308's Outcome counts the hold-out as 193/195. These cells are what is left of it.
- The cells (`make eval-show CASE=probes-numeric/t4308-gamma-density-r4-fresh.case.json`, `...t4296-...`):
  - probes-numeric/t4308-gamma-density-r4-fresh A50 `GAMMA.DIST(1.01112390509898,120.86222299463,0.0108562843982919,FALSE)`
    (x/beta = 93.1, x/a = 0.77, inside the window |x - a| <= 0.2 (x + a)): Excel 0.1000258253587687,
    ours 0.10002582535876835 (about 2.5 ulp low).
  - probes-numeric/t4308-gamma-density-r4-fresh A76 `GAMMA.DIST(31343.0713566214,24097.734513771,1.60934293687594,FALSE)`
    (x/a = 0.81, in the window): Excel 9.663478132595762E-225, ours ...764E-225 (2 ulp).
  - probes-numeric/t4296-gamma-density-r4-fresh A49 `GAMMA.DIST(5825.74385033101,8400.66908869749,0.496137034543746,FALSE)`
    (x/a = 1.40, in the window): Excel 2.211564578768812E-232, ours ...8123E-232 (1 ulp).
- All three have a scale other than 1. So do most passing cells of the two sweeps: 140 of t4308's 154 densities
  and all 160 of t4296's. The scale alone does not explain the misses. Start from T-4308's read-out
  (docs/research/t4308/read_out.py) at y = x/beta: check whether Excel's value is reachable by any double B
  through the chain. If it is not, the miss is in the chain's other operations (the scale division, the Stirling
  term, the final product), not in the exponent. These may be Excel's own noise. With three cells, a fit is not
  justified unless a fresh sweep shows a pattern.
- None is visibly off, so no campaign key. Decision 0267 does not apply (not the incomplete-beta family).

## Acceptance criteria
- [ ] The three cells are bit-exact, or each one left is listed in the Outcome with its ulp distance and the reason.
      No cell of probes-numeric/t4296-gamma-density-r4-fresh, t4308-gamma-density-r4-fresh, probes/t4290-gamma-
      window-large-pow2 or probes/t3252-gamma-prefactor-* gets worse; `make eval-baseline` adds no line.
- [ ] Any fitted or ranked choice (where the scale is divided out, an operation order) carries the R4 guards of
      reviewer-excel-fidelity check 7 (decision 0188): train and hold-out counts side by side (bit-exact, within 4
      ulp, within 15 digits) for the chosen candidate and the correctly rounded density, with the split and margin
      stated before scoring; a fresh oracle-filled sweep (probes/, at scales other than 1) drawn after the choice
      is frozen, with its result; the Gate A envelope result of that sweep (count inside, every point outside
      listed); a `FITTED` tag with its fitted range on any fitted constant; the reproducibility floor (no step
      targets differences of 4 ulp or less that vary between web and desktop or between fills); and no
      keep-matched branch.
- [ ] docs/excel-semantics/statistics.md (the T-4308 paragraph) records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA.DIST's #DIV/0! past the 2 pi a overflow (T-4340) and the density just above the window (T-4339).
- The cdf (T-4350, T-4326, T-4297).
