---
id: T-4238
title: "NEGBINOM.DIST's cdf is #NUM! on bfrac at number_s 1E+20"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`NEGBINOM.DIST(f, 1E+20, p, TRUE)` at 40 or more failures (bratio's bfrac branch) is `#NUM!` wherever Excel's is, from
a rule backed by a probe, recorded in statistics.md.

## Context
- Follow-up of T-3576, which found the L140 rule (bup's ratio product overflows) and left bfrac's cells:
  probes/t3576-negbinom-cdf-huge-s Calc!A50, A51 and A52 (`NEGBINOM.DIST(40|100|1000,1E+20,0.323212154487823,TRUE)`)
  are `#NUM!` in Excel; ours 0. bfrac at number_s 1E+15 and below is 0 at 40, 100 and 1000 failures (A22:A24,
  A36:A38, A54:A58).
- statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s (T-3553)", "Still open".
- At a0 = 1E+20 and b0 = 41, a0 + b0 rounds to a0 (ulp 16384), and TOMS brcomp's z underflows to 0, so TOMS bfrac
  returns 0 at once. Excel's bfrac prefactor is its own beta density times x·y (statistics.md "Incomplete beta L140
  and bfrac: the prefactor is Excel's own density times x·y"): the density at that shape is the first suspect.
- Code: `crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs`, `bratio::l140_overflows`'s neighbour.
- Also there: probes/t3576-negbinom-cdf-huge-s A75 (`NEGBINOM.DIST(28,999999999999999,0.99999999999999,TRUE)`, bfrac
  on the swapped call) is 1 ulp low (0.9999992473499034 against Excel's ...035): Tier B, take it only if the rule's
  work reaches it.

## Acceptance criteria
- [x] A probe brackets the number_s cut between 1E+15 and 1E+20 on bfrac (and BETA.DIST at the same shapes), and
      statistics.md records the rule or the bracket.
- [x] The engine matches probes/t3576-negbinom-cdf-huge-s A50:A52.
- [x] `make check` green; reviewers approve.

## Out of scope
- L140's overflow (T-3576, done).

## Outcome
- Rule (structural, no constant): on bratio's bfrac branch Excel's cdf is `#NUM!` exactly where fl(a0 + b0) equals a0
  or b0 (the smaller shape is absorbed). Predictions (H_absorb) were committed in `docs/research/t4238/` before the
  excel-graph fill of probes/t4238-negbinom-bfrac-huge-s: 126/126 cells matched (NEGBINOM.DIST, BETA.DIST cdf and
  density at number_s 1E+15 to 1E+20, failures 40, 100, 1000; cuts at 5E+17/6E+17, 1E+18/2E+18, 9E+18/1E+19).
- BETA.DIST's density is `#NUM!` at the same absorbed shapes on its Loader branch (both shapes above 1), which also
  fixes t3007 A20; the first draft applied it to every density and regressed t3007 A6 (`(0.1,1E+290,0.5,FALSE)` = 0,
  beta below 1), so the rule is scoped to the branch Excel's density takes there.
- Code: `bratio::shapes_absorb`, `bratio::bfrac_shapes_absorbed`; called from BETA.DIST's cdf and density and
  NEGBINOM.DIST's cdf. statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s" no longer has a "Still open".
- Eval: probes/t3576-negbinom-cdf-huge-s A50:A52 now pass (88/89; A75, 1 ulp on bfrac, stays Tier B); t4238 new
  case 126/126; t3007 A20 fixed; no regressions.
