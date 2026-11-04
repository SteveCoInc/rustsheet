---
id: T-4219
title: "GAMMA.DIST cdf at a shape near 1E8, five standard deviations below the mean: Excel is 2.5E-15 to 3.4E-15 relative off the math, ours within 2E-16 (t2982 A145, A146, unowned)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4218]
campaign: visible
---

## Goal
The incomplete gamma cdf at shapes around 1E8, a few standard deviations from the mean, gives Excel's value to
within half a unit of the 15th significant digit. The source is a model of Excel's tail arithmetic there, on both sides
of our Temme cut at 1E8, so probes/t2982-gamma-inv-huge-shape A145 and A146 leave eval/baseline/probes.txt.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 0516b489). `make exit-owners MILESTONE=M9` names these two cells
  as the only failing M9 exit cells with no open owner. They count against both the GAMMA.DIST Phase 1 rate and the
  M9 probe row. T-3070's Outcome left them to T-0971, which is done. T-3286 reported them but did not take them.
- Cells (math: 60-digit mpmath quadrature of the gamma density, computed at triage, not committed):
  - probes/t2982-gamma-inv-huge-shape.case.json (2): S!A145, A146
  - A145 `GAMMA.DIST(99950000,1E8,1,TRUE)`: Excel 2.8546421399586357E-07 (+3.35E-15, about 18 ulp), ours
    2.8546421399586256E-07 (-1.7E-16). This shape takes ours to Temme's expansion.
  - A146 `GAMMA.DIST(99950000,99999999,1,TRUE)`: Excel 2.8561234160188864E-07 (-2.5E-15, about 14 ulp), ours
    2.856123416018894E-07 (+8E-17). This shape is just below the cut, so ours takes the series or the fraction.
  - The same rows near the mean are within 15 digits: A144 `GAMMA.DIST(99990000,99999999,1,TRUE)` (z = -1), and
    T-2811's t2435 cells at the mean, 1E9 and 5E9.
- Excel is less accurate than the math here (the `visible` stage, decision 0114). Its errors have opposite signs on
  the two sides of 1E8. Find out whether Excel has a branch at 1E8 at all, or whether one formula with an error that
  grows with |z| runs on both sides. Candidates to rank, as structural models: Temme's expansion with fewer C_k
  terms, or with phi = (x - a)/a - ln(x/a) formed in doubles, not as log1pmx; erfc(sqrt(a·phi)) through Excel's own
  ERFC; the e^(-a·phi)/sqrt(2·pi·a) prefactor rounded as written; TOMS 654's own cut and its coefficients.
  docs/research/excel-kernel-playbook.md describes the method, and docs/excel-semantics/statistics.md (the "From a
  shape of 1E8 the pair is Temme's" paragraph) records what is known.
- Needs an oracle sweep: T-4218 fills shapes 3E7 to 1E10 at |z| from 0.5 to 8, with a hold-out sibling. Two cells
  cannot rank candidates, so do not fit on them alone.

## Acceptance criteria
- [ ] probes/t2982-gamma-inv-huge-shape A145 and A146 pass. Their 2 known-failure lines disappear from
      eval/baseline/probes.txt and no new lines appear in any eval/baseline/<suite>.txt (`make eval-baseline`).
- [ ] T-4218's training case: at least 90% of its cells within 15 digits of Excel. No gamma-family Excel cell
      (GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, POISSON, POISSON.DIST, GAMMA.INV, CHISQ.INV) goes from
      pass to fail, cell by cell against main.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), with each result stated in the Outcome:
  - [ ] Hold-out: counts on T-4218's hold-out sibling side by side with its training case (bit-exact, within 4 ulp,
        within 15 digits), for the chosen candidate and for the correctly rounded value. The margin is stated before
        scoring. The chosen candidate beats the correctly rounded value on hold-out.
  - [ ] Fresh sweep: after the model is frozen, a new log-uniform probes-numeric case (shapes 1E7 to 1E11, |z| up to
        10) is oracle-filled and its result is recorded here.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope (R2), with every point outside it listed.
  - [ ] `FITTED(T-4219, n_cells, range)` tag and range on every fitted constant or cut. Outside the range the code
        falls back to accurate math (today's Temme pair or series). A purely structural model needs no tag; say which
        kind it is.
  - [ ] Reproducibility floor: no step targets differences below 4 ulp (web against desktop) or bits that vary
        between fills.
  - [ ] No keep-matched branch: no band or special case whose domain is tuned to the cells that already match.
- [ ] statistics.md's huge-shape incomplete gamma entry states the model and its evidence, or records why none fits.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The gamma quantiles at these shapes (T-3272, T-3804) and the `#NUM!` from shape 1E56 (T-3855).
- The near-mode accuracy at shapes 21 to 999 (T-4000).

## Visible-parity stage
Target (decision 0114, `visible`): every cell this task owns is within half a unit of the 15th significant digit of
Excel's value. Decision 0056's time-box applies: two attempts that do not move the cells block the task as
time-boxed, with the attempts recorded. Bit-level criteria still unmet after that move to a follow-up filed with
`defer: phase-1b`.
