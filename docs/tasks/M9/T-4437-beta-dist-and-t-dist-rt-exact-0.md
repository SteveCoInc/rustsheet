---
id: T-4437
title: "BETA.DIST and T.DIST.RT exact 0 at a tiny tail: apply T-4045's L140 prefactor rule to BETA.DIST and find the earlier cut at n = 3 bup terms and in the t"
milestone: M9
lane: functions-stat
status: done
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
- T-3764 applied T-4045's rule to BETA.DIST's and BETADIST's worksheet cdf (`bratio::beta_dist_cdf`, not the quantile
  searches, so F.INV's A146 stays): t4254-beta-t-l140-zero-cut A12:A22, B12:B22, C12:C22 and D12:D22 are now Excel's 0.
  Left here: the earlier cut at n = 3 (B1:B11, D1:D11, and t4254-beta-t-zero-wide A30, B30, C30) and the t's zeros.
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

- T-4456 applied T-4045's rule to the worksheet BETA.DIST (the rule's cells A12:A22 and C12:C22 above now pass). What
  is left here is the earlier cut at few bup terms and the t's cut. T-4456's probes add evidence for both
  (statistics.md "Where the exact 0 starts (T-4045)", "More shapes"). Excel is 0 and ours is a number at:
  - probes/t4456-beta-t-zero-cut A1:A9, B1:B9 (BETA.DIST at a = 2E6, n = 5), E1:E9 and F1:F9 (T.DIST.RT at df 1E6
    and 1E8);
  - probes/t4456-beta-t-zero-reach A21, B1, B2, E21 and F9.

## Visible-parity stage
Owner of the visibly-off cells of probes/t4254-beta-t-l140-zero-cut (B1:B11, D1:D11, E1:E11, F1:F11; A12:A22, B12:B22,
C12:C22 and D12:D22 pass since T-3764)
and probes/t4254-beta-t-zero-wide (A29:A30, B28:B30, C30, F30): Excel's exact 0, or a 9th-digit difference just above
it.

## Out of scope
- F (T-4045); the df1 = 2 closed-form zeros.

## Outcome
- **Mechanism (structural model, kind (c)).** probes/t4437-beta-t-zero-bracket walks by 0.5 nats from -680 to -698.
  At (5E6, 3.5), (5E6, 4), (1E6, 3.5) and (5E6, 5.5) (n = 3 to 5), Excel's 0 starts where bup's sum W = ret·S falls
  below 2^-970 (DBL_MIN / eps): the last numbers have W = 1.10 to 1.60 times 2^-970, the first zeros 0.67 to 0.97
  times it. Code: `bup.rs` `l140_bup_sum_flushed`, asked by BETA.DIST's worksheet cdf only (`bratio::beta_dist_cdf`
  through `beta_dist_l140_zero`; F's tails and the quantile searches do not ask it).
- **First fresh sweep, as first frozen (any n): 94 of 102** (probes/t4437-beta-zero-fresh, 0.25-nat rows). Its n = 2
  family (2E6, 2.5) falsified the rule at n = 2; the n >= 3 domain was then chosen after that score (review round 1
  blocked it).
- **Second freeze and second fresh sweep (R4.2): 176 of 176.** docs/research/t4437/freeze2.md declares the rule with
  its n >= 3 domain and was pushed (e8a1e3914) before probes/t4437-beta-zero-fresh2 was filled. Eight untouched
  n >= 3 families (n 3 to 7, c 0.25 to 1, a0 1.2E6 to 9E6), 22 rows 0.05 nats apart across the edge: every family
  is a number through row 11 (0.025 nats above 2^-970) and 0 from row 12 (0.025 below), as predicted. Two n = 2
  families in the same case, out of the domain and unscored, are T-4564's evidence (the n = 2 cut lies below the
  edge and moves with a0).
- **Gate A envelope (R2) on the second sweep:** 176 of 176 in-domain cells inside (zeros exact; numbers within 1e-9
  relative of 50-digit mpmath, `t4437-beta-zero-fresh2.math.json`). Outside: only the out-of-domain J17:J22
  (Excel's n = 2 zeros, ours the math's number; T-4564).
- **Hold-out (R4.1), manual split:** the in-domain (n >= 3) L140 BETA.DIST cells of the six evidence cases
  (t4437-beta-t-zero-bracket, t4437-beta-zero-fresh, t4254-beta-t-l140-zero-cut, t4254-beta-t-zero-wide,
  t4456-beta-t-zero-cut, t4456-beta-t-zero-reach), hold-out = first byte of sha256("case!cell") mod 10 < 3, margin
  10 points. Rule (bit-exact / 4 ulp / 15 digits): train 216 / 233 / 241 of 403 (53.6% exact), hold-out 91 / 100 /
  102 of 152 (59.9%); correctly rounded math: train 0 / 1 / 6, hold-out 0 / 1 / 2. The rule beats the math on the
  hold-out and its hold-out rate is within the margin. Caveat: the split was fixed after these cells had shaped
  the rule, so the second fresh sweep is the guard that carries weight. The non-exact cells are T-4564's
  9th-digit numbers near the edge, not 0-versus-number errors.
- No `FITTED` tag: 2^-970 is DBL_MIN / DBL_EPSILON, a structural constant, and the cut was not tuned (no keep-matched
  branch). Reproducibility floor: the rule decides 0 against a number, far above 4 ulp.
- **Named cells:** t4254-beta-t-l140-zero-cut B1:B11 and D1:D11, and t4254-beta-t-zero-wide A30, B30 and C30, are
  Excel's 0. A29, B28 and B29 stay 9th-digit numbers (the criterion allows it).
- **Split to T-4564 (bracketed, unmodelled):** n = 2 (now with the second sweep's I and J families) and the t's cut,
  bracketed to 0.5 nats at df 1E7, 1E6 and 1E5. T-4564 owns t4254-beta-t-l140-zero-cut E1:E11, F1:F11,
  t4254-beta-t-zero-wide F30, and the new cases' remaining cells.
- Review round 1's design MINOR is done: one helper `bratio::l140_zero_tails` takes the zero predicate (F: prefactor
  only; BETA.DIST: `beta_dist_l140_zero`), and `l140_sum_flush_is_beta_dists_zero_at_three_terms` tests the edge.
- Follow-up from T-4518's review: T-4565 (SHEET and SHEETS of other bound values), filed here.
- Eval delta: probes fixed 45 (T-4254's 25 named cells and 20 of T-4456's zero cells), 0 regressions, 3 new cases
  (655 cells; 434 new-coverage failures, every one owned by T-4564: the n = 2 and t cells, and 8th to 9th digit
  numbers near the edge).
