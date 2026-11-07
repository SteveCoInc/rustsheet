---
id: T-4865
title: "BETA.INV at a tiny shape: when Excel's AS 109 search returns 1 - 2^-53, 2^-53, 1 or a subnormal instead of #NUM!"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.INV and BETAINV return Excel's value or Excel's `#NUM!` where the quantile is at an end of the doubles in
(0, 1) at a tiny shape: rounding to 1 at p of 1/2 or less, underflowing at p above 1/2, or with a subnormal flipped
tail at p above 1/2. Excel answers an end value at some of these points and `#NUM!` at others, and no rule that
separates them is known yet.

## Context
- Follow-up of T-3351 (PR superseding #2573), which made a quantile that rounds to 1 at p above 1/2 equal to 1 - y
  where the flipped tail's y is a normal double (122 evidenced cells, none against it), and left the rest `#NUM!`
  as before. statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)" records the evidence below.
- The three classes, Excel against ours (`#NUM!` in every one):
  1. p of 1/2 or less, quantile rounds to 1: Excel 1 - 2^-53 at 13 Gate A points (accuracy/BETA.INV/s1 and
     BETAINV/s1 A83, A86, A89, A94, A97, A98, A100, A103, A106, A109 with b from 1.7E-15 to 1.2E-3), probes/
     t3351-beta-inv-p-near-1 D197-D202, D204, D205 (1 ulp, a number already), D210 and probes-numeric/
     t3566-beta-inv-tiny-shape-and-range A166-A177, A187-A189, A193-A195. But Excel is `#NUM!` (ours too, right
     today) at s1 BETA.INV A89, A92, A95, A109 and BETAINV A83, A91 (b from 3.8E-15 to 1.6E-8), and at the tiny-p
     points A184, A189, A195, A199. Neither b, p, a nor the distance of the root from 1 separates the two sets.
  2. p above 1/2, quantile underflows: Excel 2^-53 at s1 BETA.INV A88, A97, A108 and BETAINV A85, A88, A89, A99,
     A105, A108 and t3566 A145-A153, A157-A159, A163-A165 (alpha 1.5E-15 to 1E-4), but `#NUM!` (ours too) at s1
     BETA.INV A94, A105 and BETAINV A82.
  3. p above 1/2, quantile rounds to 1 and the flipped tail's y is 0 or subnormal: Excel `#NUM!` at 154 cells and 1
     at 8: probe A199, A209-A211, C199 and probes-numeric/t3351-beta-inv-sweep A88, A89, A122 (y down to 1E-700).
     The lower-tail mirror: probe B199, B209-B211 are Excel 0 (a subnormal quantile shown as 0), ours `#NUM!`; sweep
     A175 is Excel `#NUM!` where ours is 1.5E-112, and A188 Excel 1.4E-153 where ours is `#NUM!`.
- T-3351 tried two things. (a) Applying the flip to every end (1 - 2^-53 for class 1, 2^-53 for class 2) moved
  about 60 cells to Excel but 14 Gate A points away from Excel's `#NUM!`, so it was not kept. (b) A simulation of
  AS 109 as revised (the statrs port's start, step control and `acu`) on the ported BETA.DIST kernel did not
  separate the classes either (it ended at 1 - 2^-53 or 2^-53 at most of Excel's `#NUM!` points). The outcome looks
  decided by the exact rounding of Excel's Newton steps next to 0 and 1, so the likely route is Excel's own kernel
  values at those x (probe `BETA.DIST` at x = 1 - k 2^-53 and k 2^-53 for the shapes above) before the search.
- Decision 0267 cells handed over from T-3351 (Tier A, a wrong type): t3566 A145-A153, A157-A159, A163-A177,
  A187-A189, A193-A195 (33 cells). Decision 0056's time-box applies: if two attempts do not move them, block as
  time-boxed and decision 0267 accepts them.
- Gate A: the known-failures.tsv lines of classes 1 and 2 name this task.
- Code: crates/rustsheet-engine/src/functions/statistical/beta.rs (`beta_inv_of`, `upper_quantile_near_one`,
  `lower_quantile_underflows`).

## Acceptance criteria
- [ ] Every cell listed above has Excel's value within 15 digits or Excel's error, or the task records the rule
      that would decide it as not found and blocks as time-boxed (decision 0056).
- [ ] No cell that matches Excel today moves away from it (the class 1 and 2 `#NUM!` points above included).
- [ ] Any cut chosen by scoring candidates against cells is a fit: decision 0188 R4 guards (hold-out split fixed
      before scoring, a fresh oracle sweep after the freeze commit, `FITTED` tag, `scripts/check_fitted.py`).
- [ ] The rule is in docs/excel-semantics/statistics.md; the Gate A lines that pass are removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A DBL_MIN shape (T-4663) and a huge shape (T-3862, T-4713).
- The upper-tail search at a tiny alpha and a large beta (T-4866).
