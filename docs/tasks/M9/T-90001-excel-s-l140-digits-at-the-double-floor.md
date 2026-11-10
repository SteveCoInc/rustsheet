---
id: T-90001
title: "Excel's L140 digits at the double floor on three deep tails T-4435 left visibly off (t3568 A223, A359; t4434 A295)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Three cells that T-4435 moved onto the math, and that are now visibly off Excel at decision 0267's 13-digit bar,
are within 13 digits of Excel again, or are recorded as Excel's own digits that no model reaches.

## Context
- T-4435 (decision 0341) moved these deep tails from Cephes' fraction onto the 60-digit math.
- Excel sits at its double floor on each: 0.9 to 1.7 units of |ln tail|·2^-53 off the math. At |ln tail| of about
  600 that floor is past the 13-digit bar.
- Main's kernel was not Excel's arithmetic; it only happened to sit nearer. Recovering Excel's digits needs a model
  of its own arithmetic.
- For the two BETA.DIST cells that is L140 bup with 10 or more terms at a0 of about 2500. That lies below the
  a0 > 3000 scope of T-3871 (deferred), and its many-term bup composition is T-3712's (deferred).
- For the F cell it is bgrat after the L140 bup terms (T-3209, deferred).
- This task is their active owner (decision 0267 item 4: a cell first wrong at the 13th digit reopens the work). It
  lists them instead of reopening the deferred tasks, whose scope is wider.
- Cells:
  - probes-numeric/t3568-r5-bup-fresh.case.json (1): Calc!A223 `BETA.DIST(0.75774435,2500,39.8,TRUE)`. Excel is
    1.0E-13 off the math (1.7 units), ours 4.6E-15.
  - probes-numeric/t3568-r5-bup-tune.case.json (1): Calc!A359 `BETA.DIST(0.76473,2489.3,10.3388,TRUE)`. Excel is
    6.4E-14 off the math (0.93 units), ours 8.7E-16.
  - probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json (1): Calc!A295
    `F.DIST.RT(27.1814481941,38,441015)`. Excel is 6.0E-14 off the math (1.2 units), ours 1.6E-16.

## Acceptance criteria
- [ ] Each cell is within half a unit of Excel's 13th significant digit, from a model of Excel's L140 arithmetic
      that a hold-out confirms (decision 0188 R4), with no cell of the t3568-r5-bup cases or of T-4434's case moving
      away from Excel. Or the task records why Excel's digits there are out of reach and blocks as time-boxed
      (decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy of ours, which T-4435 settled. The t3568 cells within 13 digits of Excel (Phase 1b residue,
  decision 0267).
