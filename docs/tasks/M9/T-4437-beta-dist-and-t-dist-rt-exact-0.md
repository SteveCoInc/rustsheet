---
id: T-4437
title: "BETA.DIST and T.DIST.RT exact 0 at a tiny tail: apply T-4045's L140 prefactor rule to BETA.DIST and find the earlier cut at n = 3 bup terms and in the t"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: semantics
depends_on: []
---

## Goal
BETA.DIST and T.DIST.RT are Excel's exact 0 wherever Excel is, at a tail below about 1E-288, with the cut modelled from
the mechanism (not one threshold per shape), and the numbers just above each cut kept.

## Context
- T-4254's probes (excel-graph, 2026-11-06; statistics.md "Where the exact 0 starts (T-4045)", BETA.DIST and the t):
  - probes/t4254-beta-t-l140-zero-cut walks ln of bup's prefactor ret by 1 from -698 (row 1) to -719 (row 22) on L140
    bup+bgrat.
    - A (a, b) = (2E6, 10.5) and C (2E6, 10) follow T-4045's rule exactly: a number at -708 (row 11), 0 from -709
      (row 12).
    - B (5E6, 3.5) and D (5E6, 4) are 0 at every row.
    - E and F (T.DIST.RT at df 1E5 and 1E7, walked in ln of the true tail) are 0 at every row.
    - Ours is a number at A12:A22, B1:B22, C12:C22, D1:D22, E1:E11 and F1:F11, where Excel is 0.
  - probes/t4254-beta-t-zero-wide walks by 10 from -400 (row 1) to -690 (row 30).
    - At n = 3 bup terms (A (5E6, 3.5), B (5E6, 4), C (1E6, 3.5)) Excel is a number at -680 (row 29) and 0 at -690
      (row 30), 20 to 30 nats above ln(DBL_MIN).
    - D (5E6, 10.5) is a number at every row.
    - T.DIST.RT: E (df 1E5) is a number to -690. F (df 1E7) is a number at -680 and 0 at -690 (F30). With the first
      case, E's cut lies in (-698, -690) of the true tail.
    - Ours is a number where Excel is 0 at A30, B30, C30 and F30. A29, B28 and B29 differ from Excel in the 9th digit,
      just above the cut.
  - At the n = 3 brackets, ln of bgrat's increment D = I_x0(a0, c) is -687 (number) and -697 (0), but D (5E6, 10.5) is
    a number at ln D -697. So neither D alone nor the tail explains the earlier cut. Suspects: bup's terms at small n,
    or bgrat's own prefactor `u` and its error exit.
- The t never reaches L140: its kernel is K(df/2, 1/2), and b = 1/2 takes `small_shape_branch` (`bratio::dispatch`).
  Its 0 needs its own mechanism: Bgrat or W1Bpser at a0 = 1/2 after the swap.
- Code: `bratio::f_closed_or_ported_tails` and `l140_prefactor_zero_tails` (F only today); `bup.rs`
  `l140_bup_prefactor_underflows`; `beta_dist_lower_tail`; `t_kernel_tails`. BETA.DIST's cdf also feeds F.INV's search,
  and a 0 there moves `F.INV(1.43795578761177E-262,107702,58)` (accuracy/F.INV/s1 A146): check it before applying the
  rule to BETA.DIST.

## Acceptance criteria
- [ ] Every cell of probes/t4254-beta-t-l140-zero-cut and probes/t4254-beta-t-zero-wide named above matches Excel's 0 or
      number (the 9th-digit cells may stay within the family's digits bar).
- [ ] The cut at n = 3 bup terms and the t's cut are bracketed by a further probe to about 1 nat and modelled as a
      mechanism (decision 0188 R4 guards if anything is chosen by scoring), recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Owner of the visibly-off cells of probes/t4254-beta-t-l140-zero-cut (A12:A22, B1:B22, C12:C22, D1:D22, E1:E11, F1:F11)
and probes/t4254-beta-t-zero-wide (A29:A30, B28:B30, C30, F30): Excel's exact 0, or a 9th-digit difference just above
it.

## Out of scope
- F (T-4045); the df1 = 2 closed-form zeros.
