---
id: T-4456
title: "BETA.DIST takes T-4045's L140 prefactor zero rule (T-4437's first part)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The worksheet BETA.DIST cdf is Excel's exact 0 wherever T-4045's rule applies: L140 bup+bgrat with bup's prefactor ret
below DBL_MIN. F.INV's search keeps today's kernel.

## Context
- Split from T-4437. T-4437 keeps the earlier cut at few bup terms and the t's cut.
- Evidence: probes/t4254-beta-t-l140-zero-cut A12:A22 and C12:C22 (T-4254, PR #3469) are 0 from ln ret -709. A
  parked branch from T-4254 (agent/functions-stat/T-4254-beta-t-zero-edge-probe, PR #3464 closed as a duplicate) has
  two more filled probes: n = 25 and 29 bup terms at a = 5E6, and the earlier cut against b, a and df. It also has the
  engine change: `bratio::l140_prefactor_zero_tails` asked from `beta.rs`, so `beta_dist_lower_tail` stays unchanged
  for F.INV's search (accuracy/F.INV/s1 A146).

## Acceptance criteria
- [ ] BETA.DIST's cdf is 0 at the rule's cells of probes/t4254-beta-t-l140-zero-cut, and at those of the salvaged
      probe; unit test; accuracy/F.INV/s1 A146 is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
