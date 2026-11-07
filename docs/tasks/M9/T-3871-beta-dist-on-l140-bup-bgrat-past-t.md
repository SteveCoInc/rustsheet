---
id: T-3871
title: "BETA.DIST on L140 bup+bgrat past T-3561's caps (n >= 10, a0 > 3000), and the statistics.md scope of T-3561's port"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3561]
defer: phase-1b
---

## Goal
BETA.DIST's cdf on bratio's L140 bup+bgrat branch is within 15 digits of Excel past T-3561's two caps: n >= 10 bup
terms (b0 from 10 to 40) and a0 > 3000. docs/excel-semantics/statistics.md also states the scope of T-3561's port
correctly.

## Context
- T-3561 (PR #2887) ported T-3568's n-term composition to BETA.DIST's cdf, capped at `MAX_BUP_TERMS` = 9 and
  `MAX_BUP_A0` = 3000 (`bratio/bup.rs`). Past the caps today's kernel answers.
  - The n cap: T-3568's residual grows with the binade of (n - 1)·|ln y0| from n = 10 (r5-bup RESULTS.md section 5).
    Every t3568-r5-bup cell the uncapped port took from within 15 digits to visibly off had n >= 19 (30 cells).
  - The a0 cap: T-3568's probe range. At a0 = 5E9 the port took probes-numeric t2528-f-dist-huge-df2 A4 and A16 from
    within 4 ulp to visibly off, while its a0 = 5E8 cells (A24, A28, A32, A36, A40) became exact.
- No task owned these cells: T-3712 is F, BINOM and NEGBINOM only, and lists the n >= 10 leftover as out of scope.
  (T-3561's Excel-fidelity review, MINOR.)
- Doc fixes from the same review:
  - statistics.md's "L140 bup+bgrat with n >= 2 bup terms" bullet says the engine takes the composition for "BETA.DIST's
    cdf and BETA.INV's search". The search stays on today's kernel there (`beta_search_tails`, T-3867), and T-3712's
    Context says the same wrong thing. Make both say BETA.DIST's cdf only.
  - The bullet should name the two caps next to the density and e^-650 fallbacks.

## Acceptance criteria
- [ ] statistics.md and T-3712 state that the port is BETA.DIST's cdf only and name both caps.
- [ ] For n in [10, 39] and for a0 > 3000, either a form that keeps every cell of the t3568-r5-bup cases and
      t2528-f-dist-huge-df2 within 15 digits where it was before, with the per-case deltas in the Outcome, or a
      probe-backed finding in statistics.md of why not, with the cells left listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV's search (T-3867), F, BINOM and NEGBINOM (T-3712), and Excel's density to the bit (T-3569).

## Deferred by decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) parks this task until Phase 1 exits. It is work on the incomplete-beta family whose cells are first wrong only at the 14th or 15th significant digit, or a probe or port that serves only that work. In those regions Excel itself is not correctly rounded (on a sample of 2,026 visibly-off cells, Excel was correctly rounded on 1%, with a median error of 48 to 330 ulp). Matching those digits means reproducing Excel's rounding noise. Decision 0267 accepts those cells. Reopen the task only if a cell it owns is first wrong at the 13th digit or earlier.
