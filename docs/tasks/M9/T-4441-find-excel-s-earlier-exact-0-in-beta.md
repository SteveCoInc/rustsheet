---
id: T-4441
title: "Find Excel's earlier exact 0 in BETA.DIST at few L140 bup terms and in T.DIST.RT (tails near 1E-295 read 0 above DBL_MIN)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine returns Excel's exact 0 where Excel's BETA.DIST cdf (L140 bup+bgrat at few bup terms) and T.DIST.RT read 0
although the tail is a normal double and T-4045's prefactor rule (bup's ret below DBL_MIN) does not apply.

## Context
T-4254 (statistics.md "Where the exact 0 starts (T-4045)", BETA.DIST and the t) found a second, earlier 0. Ours is the
math's number at every cell below; Excel's is 0.
- probes/t4254-beta-t-zero-cut A1:A9 and B1:B9: `BETA.DIST(x,2E6,5.5|6,TRUE)` at ln ret -700 to -708 (true tail 4E-295
  down to 7E-299), n = 5 bup terms, c = 1/2 and 1.
- probes/t4254-beta-t-zero-cut E1:E9 and F1:F9: `T.DIST.RT(t,1E6|1E8)` at ln of the true tail -700 to -708.
- probes/t4254-beta-t-zero-reach A21, B1, B2, E21 and F9: `BETA.DIST(0.99964836251088,2E6,5.5,TRUE)` (4.1E-295),
  `BETA.DIST(0.999653364345583,2E6,2.5|3.5,TRUE)` (1.0E-297, 2.8E-295) while b = 5.5 at the same x is a number
  (8.5E-291); `T.DIST.RT(37.3080613695331,1E6)` (9.9E-305) while E20 (2.17E-300) is a number; and
  `T.DIST.RT(37.0262990446347,1E8)` (2.17E-300) while df 5 to 1E6 at the same tail are numbers.
- Leads, untested: for BETA.DIST the cut sits between 8.5E-291 and 2.8E-295 in w at every shape probed, around
  DBL_MIN/eps = 2^-970 (1.0E-292); look at bgrat's `l = w/u` and its break test, and bratio's handling of a w below
  DBL_MIN/eps. The t takes bratio's Bgrat branch at a0 = 1/2 (`bratio::dispatch`), not L140, and its cut moves with df.
- Do not guess: probe a fine walk (steps of about 0.25 in ln w) across 1E-291 to 1E-296 at three shapes and n from 1
  to 9, and the t at four dfs, then fit one rule with a hold-out (decision 0056).

## Acceptance criteria
- [ ] A probe pins each edge to one step; statistics.md records the rule.
- [ ] The engine is Excel's 0 on the cells above and on the probe's hold-out, and no number of
      probes/t4254-beta-t-zero-reach becomes 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
