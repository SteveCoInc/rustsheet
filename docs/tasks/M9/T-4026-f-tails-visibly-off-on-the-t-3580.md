---
id: T-4026
title: "F tails visibly off on the T-3580 fresh sweep: deep tails at general dfs"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The F cdf and upper tail match Excel within 15 significant digits on the 45 cells of T-3580's fresh sweep that are
visibly off, or each remaining one is explained by an open, cited Excel quirk.

## Context
- Found by T-3580 (docs/research/t3580/RESULTS.md). Case probes-numeric t3580-f-fresh (2,400 cells, excel-graph):
  44 of the 45 are its `gen` region (d1, d2 log-uniform in [1, 500], 1/3 non-integer literals) in deep tails
  (1E-7 to 1E-303), 0.5-30 units of the 15th digit off; one (A481, `F.DIST.RT(26.8242,1,16)`) is d1 = 1.
  Every one is inside Gate A's envelope (within 1e-9 of the 50-digit math) and was already off on main before T-3580,
  so these are the kernel's open branches (bpser at a, b > 1, bfrac, L140; statistics.md's bratio entries), not
  T-3580's arguments.
- Cells of t3580-f-fresh: A481 A903 A905 A923 A933 A935 A948 A1003 A1009 A1021 A1029 A1042 A1053 A1056 A1062 A1086
  A1097 A1098 A1106 A1129 A1144 A1148 A1161 A1166 A1169 A1176 A1184 B912 B918 B919 B945 B949 B959 B983 B1012 B1036
  B1063 B1087 B1102 B1105 B1108 B1118 B1164 B1185 B1196.
- First step: label each cell with its bratio branch on F's arguments (`bratio::dispatch` on `fisher.rs`
  `f_beta_arguments`) and route it to the task that owns that branch, if one exists.

## Acceptance criteria
- [ ] Each of the 45 cells is within 15 digits of Excel, or named by an open task that owns its branch, with the
      branch recorded here.
- [ ] No cell of t3580-f-fresh or t3567-r5-fdf1-hold newly visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.INV, F.INV.RT and FINV (`f_quantile_tail`).
- The bup misses of decision 0263 (bup's arithmetic on F's arguments).
