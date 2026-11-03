---
id: T-3913
title: "Probe bfrac's Lentz region through F.DIST and BINOM.DIST (non-BETA.DIST callers, T-3812)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence shows whether the callers of bratio other than BETA.DIST (F.DIST, F.DIST.RT, FDIST and BINOM.DIST)
match Excel on bfrac's Lentz region (T-3812). If they do not, the engine keeps them on TOMS's recurrence.

## Context
- T-3812 gates `lentz_fraction` (crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs `lentz_region`)
  on bratio's dispatch only. Every caller that reaches bfrac with a, b in [40, 5000], the unswapped call and z in
  [-8, -0.2] sd therefore takes the loop. T-3571's evidence is BETA.DIST only, where y = fl(1 - x). F.DIST passes
  its own complement v = df2/(df1·x + df2) (fisher.rs `f_beta_arguments`), and that complement feeds β's n·(y + 1).
- Follow-up of T-3812's excel-fidelity review (MINOR). Gated eval shows no change for these callers. Before
  T-3812 they ran TOMS's recurrence, which is just as unprobed on this region.
- Suggested probe (excel-probe skill, probes-numeric): F.DIST(x, d1, d2, TRUE) and F.DIST.RT at d1, d2 log-uniform in
  80..10000, with x chosen so that the kernel point (a, b) = (d1/2, d2/2) lies at z in [-8, -0.2] on the unswapped
  call; BINOM.DIST(k, n, p, TRUE) with n - k and k + 1 in [40, 5000] on the same z band.
- statistics.md "Incomplete beta bfrac is TOMS's fraction in a Lentz loop": its Ours line names these callers as
  UNVERIFIED.

## Acceptance criteria
- [ ] A probes-numeric case (excel-graph) scores each caller on the region with the Lentz loop and with TOMS's
      recurrence: bit-exact, within 4 ulp, and within 15 digits.
- [ ] Either the callers keep the loop (the scores show it is no worse) or the gate excludes them, with the counts
      recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST itself (T-3812, T-3813), the density (T-3244).
