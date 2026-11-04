---
id: T-4073
title: "BETA.DIST and BETA.INV at a first shape of 3000 to 1E7 on L140 bup+bgrat: oracle probe, and fix statistics.md's stale F sentence (T-3754 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence covers BETA.DIST's cdf and BETA.INV where L140 bup+bgrat now takes a first shape a0 of 3000 to 1E7
(T-3754 raised `MAX_BUP_A0` from 3000), and statistics.md says without contradiction which callers take that composition.

## Context
- T-3754 (PR #3122) raised the cap in `crates/rustsheet-engine/src/functions/statistical/bratio/bup.rs` (`MAX_BUP_A0`,
  `l140_bup_bgrat`). Its evidence is F only: the 50-digit grid in docs/research/t3754 and 52 F/F.INV oracle cells that
  move into 15 digits. The same branch (`Branch::L140BupBgrat if half_bgrat`) also serves BETA.DIST's cdf and
  BETA.INV's search. The fidelity review on #3122 found no eval cell with a BETA.DIST shape between 3000 and 1E7.
- docs/excel-semantics/statistics.md, bullet "L140 bup+bgrat with n >= 2 bup terms": the sentence "F, BINOM and NEGBINOM
  stay on today's kernel (T-3712)" is stale. F takes the composition through its half_bgrat caller
  (`bratio::f_closed_or_ported_tails`); BINOM and NEGBINOM do not (`kernel_tails`). The new sub-bullet below it, from T-3754,
  says F takes it, so the two contradict each other.
- The grid in docs/research/t3754 starts at d2 = 1E5 (a0 = 5E4) and stops at x = 2. Adding d2 in {1E4, 3E4} and x in {3, 10}
  would cover the cap's lower range and the deeper tails.

## Acceptance criteria
- [ ] New probes-numeric case `t4073-beta-l140-large-a0.case.json`, filled with excel-graph (excel-probe skill):
      `BETA.DIST(x, a, b, TRUE)` with a log-uniform in [3000, 1E7], b in {2, 2.5, 3, 4, 7.5, 10, 20, 39}, x with
      x chosen so that bratio swaps to a0 = a and x0 > 0.7 (tails from about 1E-1 to 1E-12), plus `BETA.INV` at the
      cdf values for a subset. Seeded and deterministic, at most about 600 cells, with the math sidecar (SPEC-0029).
- [ ] The Outcome reports, for the engine on main, cells bit-exact, within 4 ulp and within 15 digits of Excel, and the
      largest relative error against the math. Every visibly-off cell has an open owner task.
- [ ] statistics.md's stale sentence names the callers that take the composition (BETA.DIST, BETA.INV's search, the
      Student t, F) and the ones that do not (BINOM, NEGBINOM, F.TEST), with the a0 <= 1E7 cap.
- [ ] docs/research/t3754/grid.py gains d2 in {1E4, 3E4} and x in {3, 10}, and its README table is re-run; every stratum
      stays under 1E-14, or the Outcome names the stratum and files a task for it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the cap or the kernel (a new task, if the probe shows BETA.DIST visibly off).
