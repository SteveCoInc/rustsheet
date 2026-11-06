---
id: T-4442
title: "F.INV at a p inside Excel's L140 exact-0 region: Excel returns a root where its own F.DIST is 0 (accuracy/F.INV/s1 A146)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know, and if it is a rule, match, what Excel's F.INV returns for a p whose root lies where Excel's own F.DIST cdf is 0.

## Context
probes/t4254-beta-t-zero-cut G1 repeats accuracy/F.INV/s1 A146: `F.INV(1.43795578761177E-262,107702,58)` is
0.02386747293188499 in Excel; the math's root is 0.0400613619170450. Excel's F.DIST at 107702, 58 is 0 from x = 0.02 to
0.04 (probes/t4254-beta-t-zero-cut G2:G12, among them at Excel's root and at the math's root, where ln of bup's prefactor
is -1194 and -712.5, T-4045's exact 0) and a number from 0.0425 (G13, 1.8E-245). So Excel's search does not cross its
cdf at p there: it returns a point inside the flat 0 (statistics.md "Where the exact 0 starts (T-4045)", T-4254). Ours
(`inverse_beta.rs` `beta_lower_crossing`, over `bratio::beta_dist_lower_tail`, which keeps today's kernel there) is the
math's root. T-1871 owns F.INV's search in general.
- Probe F.INV at several p inside the zero region (p from 1E-300 to the smallest number the cdf reaches) at two (df1, df2)
  and fit what Excel's search returns (a bisection midpoint, a fixed start).

## Acceptance criteria
- [ ] statistics.md records what Excel returns there, with probe evidence.
- [ ] The engine matches it on the probe, or the task records why it cannot (decision record).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
