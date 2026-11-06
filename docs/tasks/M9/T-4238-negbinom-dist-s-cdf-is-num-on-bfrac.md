---
id: T-4238
title: "NEGBINOM.DIST's cdf is #NUM! on bfrac at number_s 1E+20"
milestone: M9
lane: functions-stat
status: todo
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
- [ ] A probe brackets the number_s cut between 1E+15 and 1E+20 on bfrac (and BETA.DIST at the same shapes), and
      statistics.md records the rule or the bracket.
- [ ] The engine matches probes/t3576-negbinom-cdf-huge-s A50:A52.
- [ ] `make check` green; reviewers approve.

## Out of scope
- L140's overflow (T-3576, done).
