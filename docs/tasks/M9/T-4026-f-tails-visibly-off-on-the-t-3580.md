---
id: T-4026
title: "F tails visibly off on the T-3580 fresh sweep: deep tails at general dfs"
milestone: M9
lane: functions-stat
status: done
priority: P1
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
- [x] Each of the 45 cells is within 15 digits of Excel, or named by an open task that owns its branch, with the
      branch recorded here.
- [x] No cell of t3580-f-fresh or t3567-r5-fdf1-hold newly visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.INV, F.INV.RT and FINV (`f_quantile_tail`).
- The bup misses of decision 0263 (bup's arithmetic on F's arguments).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 6 probes-numeric cells that no open task listed: 6 first wrong at the 11th significant digit or earlier.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3567-r5-fdf1-hold.case.json (2): Calc!B1003, B1307
- probes-numeric/t3580-f-fresh.case.json (4): Calc!B723, B751, B824, B848

- [x] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).

## Outcome
- **Tier A (decision 0267), all 6 fixed.** Each was Excel's 0 against our subnormal cdf (8.4E-309 to 1.2E-312). Graph
  shows a kept subnormal as 0 too (decision 0042), so probes/t4026-f-cdf-subnormal (210 cells, excel-graph,
  2026-11-05; generator docs/research/t4026/mkprobe.py) put a `=(..)=0` companion on each: TRUE for all 6, so they
  are true zeros. Walking ln of the cdf from -700 to -716 at (316, 1) and (40, 1), whose bpser prefactors 1/a0
  differ eightfold, both keep 3.3E-308 and give 0 at 2.0E-308: the cut is DBL_MIN on the final result
  (statistics.md "F.DIST's cdf flushes a subnormal result to 0 (T-4026)"). `fisher.rs` `f_dist_of` now flushes the
  cdf with the existing `arithmetic::flush_subnormal`; unit test `test_fn_f_dist_cdf_flushes_a_subnormal_result`.
  Not a fit: one structural rule read off the probe, no constant scored against oracle cells.
- **The 45 listed cells, branch-labelled and routed.** All 45 are still visibly off, at the 14th or 15th digit only
  (0.5 to 13.3 units of the 15th), so they are Tier C/D, which decision 0267 cuts. Branch by `bratio::dispatch` on
  `f_beta_arguments` at the truncated dfs: bpser with both shapes above 1 (22), bfrac (18), L140 bup+bgrat (B1063,
  B1185, B1196), w = bpser at b0 <= 1 (A481, df1 1) and the df-of-2 closed form (A1042). The 18 bfrac cells are
  listed in T-3323 (bfrac bit-level); the other 27 in T-3592 (last-digit stat owner), both `defer: phase-1b`.
- **Eval delta:** probes-numeric t3580-f-fresh 1,867 to 1,871 passing, t3567-r5-fdf1-hold 2,879 to 2,881; the new
  probe 174 of 210 (the 36 misses are 1-2 ulp NumericUlp cells of rows 7-105, none visibly off). No cell newly
  visibly off: the change touches only a cdf below 2.2E-308.
- Follow-up: T-4282 (sonnet) probes F.DIST.RT, FDIST and BETA.DIST at a subnormal result the same way.
