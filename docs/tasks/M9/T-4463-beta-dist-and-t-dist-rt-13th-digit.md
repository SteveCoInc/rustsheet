---
id: T-4463
title: "BETA.DIST and T.DIST.RT 13th-digit residue in tails near 1E-290 at a huge first shape or df (t4456 probes)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Ours is Excel's value to the family's digits bar (decision 0267: within half a unit of the 13th significant digit) at the
deep-tail cells below, or the task records why a cell stays off.

## Context
- T-4456 salvaged two excel-graph probes of BETA.DIST and T.DIST.RT tails between about 1E-300 and 1E-209 at a huge
  first shape or df: L140 bup+bgrat for BETA.DIST, and bgrat at a0 = 1/2 for the t. The zeros there belong to T-4045,
  T-4456 and T-4437. The numbers match Excel to 12 or 13 digits, but these cells are 7.6E-15 to 6.5E-13 relative off
  (ours minus Excel over Excel), which is outside the 13-digit bar:
  - probes/t4456-beta-t-zero-cut (6): C1, C3, C8, D1, D4 (BETA.DIST at a = 5E6, b = 25.5 and 30, n = 25 and 29 bup
    terms, the fallback kernel past the n <= 9 port) and G14 (`F.DIST(0.045,107702,58,TRUE)`).
  - probes/t4456-beta-t-zero-reach (24): A20, B3, B4, B5, C2, C3, C4, C5, C6, C7, C8, C9, D1, D2, D3, D4, D7 (BETA.DIST
    at ln ret -690, a from 1E4 to 5E7) and E17, E18, E19, E20, F6, F7, F8 (T.DIST.RT at df 1E4 to 1E6).
- Code: `bratio` `l140_bup_bgrat`, with MAX_BUP_TERMS 9 and MAX_BUP_A0 1E7, so the fallback kernel takes most of these.
  The t's tail is `t_kernel_tails` (bgrat at b = 1/2). Compare with 50-digit mpmath first: decision 0188 R3 applies if
  Excel is the one that is off.

## Acceptance criteria
- [ ] Each cell above is within the family's 13-digit bar of Excel, or stays a known failure with its reason recorded
      in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
