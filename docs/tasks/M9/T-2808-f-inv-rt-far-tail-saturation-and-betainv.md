---
id: T-2808
title: "F.INV.RT far-tail saturation and BETAINV's underflow and 0.5 quirk (T-2435 split)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
F.INV.RT gives Excel's saturated value in the far tail, and BETA.INV/BETAINV Excel's `#NUM!` where the
quantile underflows (and its 0.5 at a tiny alpha with a high probability).

## Context
Split out of T-2435 (2026-10-26), which settled the BINOM, normal-density, BETA.DIST-endpoint and
F-cdf rules of the same gen-m9 sweep and left these two search-internal ones. Both rules and their
probe evidence are in docs/excel-semantics/statistics.md, "F.INV.RT saturates at
3.696066238512854E+150/df1 in the far tail (T-2435)" and "BETAINV is #NUM! where its quantile
underflows, but 0.5 at a tiny alpha and a high p (T-2435)"; the cells are
probes/t2435-dist-extremes A68:A81, A108:A112, A115:A118 (known failures), their `(...)=0` companions in column B
(cell list below), plus the owned gen-m9 cells below. The F.INV.RT constant 3.696066238512854E+150 is df1-only (half it at df1 = 2), so it
looks like the search's own upper bracket: find what it is before implementing a cap, and read
docs/research/excel-kernel-playbook.md first. BETAINV's 0.5 is Excel's search stopping at its
start, which the crossing refinement in functions/statistical/inverse_beta.rs does not model.
Code: functions/statistical/fisher.rs (`f_quantile`), beta.rs (`beta_quantile`), inverse_beta.rs.

Column-B companions (gap triage 2026-10-31: visibly off, Excel `#NUM!` or FALSE against ours FALSE or `#NUM!`, and no
open task named them; they pass with their column-A cells):
- probes/t2435-dist-extremes.case.json (9): S!B68, B69, B77, B79, B110, B115, B116, B117, B118

Owned gen-m9 cells (T-2435 split, 2026-10-26): 4 known-failure lines in eval/baseline/gen-m9.txt.
- gen-m9/3e98b26cf6713e77 Calc!A151 `=BETAINV(1E-20,1E-300,30)`: Excel #NUM!, ours 3.333333333333333e-302
- gen-m9/abc2939ebd1e2365 Calc!A11 `=CHIDIST(F.INV.RT(1E-200,1,1.999),10)`: Excel 0.0, ours #NUM!
- gen-m9/a612e89321a6df9c Calc!A23 `=GAMMALN.PRECISE(F.INV.RT(1E-20,3,9))`: Excel 1087085.7102783846, ours ...853
- gen-m9/01523d247594952e Calc!A377 `=SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`: Excel -1987062826671.328, ours -1987062826671.3352 (the F.INV.RT far tail, T-2491 hand-off)
- Gate A (T-3578): accuracy/BETA.INV/s1 A115, A118, A125, A128 and A138 (Excel 0.5 with a shape of DBL_MIN) are known
  failures owned by this task; remove their lines from eval/accuracy/known-failures.tsv when they pass.

## Acceptance criteria
Narrowed 2026-11-06 (owner rule: "do not overfit any function values"; see the Outcome): this task keeps the
underflow `#NUM!`; F.INV.RT's saturation moved to T-4662 and BETA.INV's 0.5 to T-4663, both `defer: phase-1b`.
- [x] The underflow cells pass: gen-m9/3e98b26cf6713e77 A151 and probes/t2435-dist-extremes A68, A69, A110 with
      B68, B69, B110, with no new known-failure lines in any baseline.
- [x] The cells that cannot pass without a fitted value are named with evidence in the statistics.md entries above
      and owned by T-4662 (F.INV.RT: A77, A79, A115:A118, B77, B79, B115:B118, gen-m9 abc2939ebd1e2365 A11,
      a612e89321a6df9c A23, 01523d247594952e A377) and T-4663 (A73, A111 and the 10 Gate A lines).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The BINOM, normal-density, BETA.DIST-endpoint and F-cdf rules (T-2435, done).
- GAMMA.DIST at a huge shape (T-1891).

## Outcome
Implemented: BETA.INV and BETAINV are `#NUM!` where the quantile underflows, for p up to 1/2
(functions/statistical/beta.rs `lower_quantile_underflows`). Structural justification: the quantile lies below
DBL_MIN exactly when I_x(a, b) at x = DBL_MIN is at least p, and at that x the incomplete beta is its series'
leading term x^a/(a B(a, b)) to the double (next term a (1 - b) x/(a + 1) relative). That is the math plus the
existing flush-to-`#NUM!` rule, with no constant fitted; it agrees with every probed `#NUM!` and number in
statistics.md's entry (A68:A76, A108:A112). The old search could not see it at alpha 1E-300, where every cdf rounds
to 1, and stopped at 3.333333333333333E-302. Unit test `a_quantile_below_the_smallest_normal_double_is_zero`.

Not implemented, with the reason (statistics.md entries updated):
- F.INV.RT's 3.696066238512854E+150/df1: no structural derivation found. The constant is no simple closed form of
  DBL_MAX, epsilon or a power of two (2^500 x 1.1291247153936121, sqrt(DBL_MAX)/3627.59, e^346.695), and the df1 = 2
  value is not exactly half (ratio 1.0000010000006678 to C/2). An AS 109 replay with acu = 1E-300 stops where the
  Newton correction falls below 1E-150, giving the order and the 1/df1 scaling, but depends on df2 (0.64 to 1.36
  times C) where Excel's does not. Copying the constant would be a fit, so it stays a known failure: T-4662.
- BETA.INV's 0.5: the candidate rule (AS 109's first approximation is 1 in its own orientation and the search is
  reset to the midpoint) fits 12 of 13 cells, but BETAINV/s1 A125 is `#NUM!` from the same start, and neither the
  start nor a non-moving first step separates it from A135 (0.5). With the candidate refuted in-sample there was no
  rule to commit predictions for, so no probe was filled: T-4663. Its 10 Gate A lines moved from T-2808 to T-4663
  in eval/accuracy/known-failures.tsv.

Eval delta: gen-m9 known failures 1768 -> 1767 (3e98b26cf6713e77 A151), probes 8364 -> 8358
(t2435-dist-extremes A68, A69, A110, B68, B69, B110), 0 regressions; Gate A `rustsheet-eval accuracy` 0 failing
(T-4663: 10 known).
