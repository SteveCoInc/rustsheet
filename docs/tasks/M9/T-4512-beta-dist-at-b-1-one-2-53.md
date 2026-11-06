---
id: T-4512
title: "BETA.DIST at b = 1: one 2^-53 step of w left at |z| 3 to 9 after T-3764's 1 - EXP switch (t3764 fresh A120 at the 12th digit)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`BETA.DIST(x, a, 1, TRUE)` at x > 0.9 and `F.DIST.RT`/`F.DIST` at d1 = 2 match Excel where w = 1 - (1 - x0)^b0 is still
one 2^-53 step off after T-3764, starting with the one visibly-off cell of probes-numeric
t3764-shape-one-complement-fresh: Calc!A120 (`BETA.DIST(0.914472771402,102.725,1,TRUE)`, Excel 0.00010262856413656429,
ours 0.00010262856413667532, 12th digit).

## Context
- T-3764 (statistics.md "Incomplete beta at a shape of 1 outside the closed forms"): from |z| = 1, z = b0·ln1p(-x0),
  Excel's w is `1 - EXP(z)`; that matches all 508 t3568-r5-bup cells (|z| 3.9 to 215) and most of the fresh sweep.
- What is left on the fresh sweep (excel-graph, 2026-11-06): 9 BETA.DIST cells (A96, A116, A120, A359, A361, A412,
  A433, A436, A451) and 8 F.DIST.RT cells (B4, B71, B359, B363, B377, B395, B412, B449), w1 from 1E-4 to 0.05, BETA.DIST
  |z| 3.0 to 9.2, each one 2^-53 step of w off. Only A120 is visibly off (decision 0267's 13-digit bar); the others are
  `residue13`.
- Suspects: Excel's log1p near a tie, multiplied by b0 (statistics.md "Excel's log1p is a 14-term atanh series"), or
  EXP's own last bit at these z (numbers.md "EXP", model V6). Read docs/research/excel-kernel-playbook.md first.
- Decision 0056's time-box applies; decision 0267 accepts the cells if two attempts do not move them.

## Acceptance criteria
- [ ] A120 within 13 significant digits of Excel, or the task time-boxed with the attempts listed.
- [ ] No cell of t3568-r5-bup-tune, -hold, -fresh or t3764-shape-one-complement-fresh column A or B that matches today
      moves off.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Owner of probes-numeric/t3764-shape-one-complement-fresh Calc!A120 (decision 0245).

## Out of scope
- The 6 small-|z| contrary cells of T-3886.
