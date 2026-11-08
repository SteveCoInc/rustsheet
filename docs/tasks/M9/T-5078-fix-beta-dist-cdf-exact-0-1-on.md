---
id: T-5078
title: "Fix BETA.DIST cdf exact 0/1 on the direct bgrat branch and BETA.INV at p<=0.9 for a tiny alpha, large beta (T-5066 probe)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
BETA.DIST's cdf and BETA.INV match Excel at a tiny alpha and a large beta on bratio's direct bgrat branch, where ours returns an exact 0 or 1 (cdf) or a quantile Excel does not (INV at p of 0.9 and below).

## Context
T-5066 probed this (probes/t5066-beta-dist-direct-bgrat, 116 cells, excel-graph, docs/excel-semantics/statistics.md "BETA.INV's search at a tiny alpha and a large beta"). Owned cells (visibly off):
- probes/t5066-beta-dist-direct-bgrat.case.json: Calc!A5:A7, A9, A13, A21:A23, A25, A29, A37:A39, A45, A53:A56, A61, A105:A107, A109:A111, A113:A116, A83:A84, A87:A88, A91:A92, A95:A96, A99:A100, A103:A104.
- BETA.DIST (A5:A7 etc.): at beta 171.17 ours is an exact 0 at x = 1E-5 and an exact 1 at x >= 1E-3 (Cephes kernel); Excel is 0.97 / 0.9932 / ...; at beta 500 and 2000, x = 1E-5, ours differs from Excel at the 4th digit (A9, A13, A25, A29, A45, A61).
- BETA.INV (A83:A104): at p of 0.9 and 0.75 Excel's root is 1.1102230246251565E-16 (1 ulp of 1 / 2^-53) where the true root is far below or ours stops at a band edge. Excel's behaviour there needs its own reading (the table in statistics.md).
Moving BETA.DIST onto direct bgrat everywhere is T-3933 (deferred by decision 0267); this task takes only the points where ours is grossly off.

## Acceptance criteria
- [ ] The cells named in Context match Excel (or are within the accepted tolerance) and are in the baseline as passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
