---
id: T-4255
title: "Identify Excel's large-shape gamma prefactor inside its window (0.668a to 0.99a, 1.05a to 1.45a) and POISSON's far upper tail form"
milestone: M9
lane: research
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The model Excel's large-shape (a >= 20) gamma prefactor uses where it is neither TOMS 654's `rcomp` nor its
ratio form (T-2638) is identified, or ruled out form by form, so the engine can take it there.

## Context
T-3252 (docs/excel-semantics/statistics.md, "Incomplete gamma: accurate kernel", the T-3252 paragraph) found:
- Below the mode, from 0.668a to 0.99a, Excel is neither form. Each is 0 to 300 ulp off, with no side consistently
  closer: probes/t3252-gamma-prefactor-holdout-2 A5:A12, A17:A24, A29:A36; probes/t3252-gamma-prefactor-holdout
  A5:A14, A19:A28, A33:A42; probes/t3252-gamma-prefactor-window A4:A10, A24:A30, A44:A50. At x <= 0.664a the ratio
  form is exact.
- The post-freeze sweep probes/t3252-gamma-prefactor-sweep (x/a 0.55 to 0.75, shapes 30 to 500) has 10 visibly-off
  cells, all in that band or at its edge: A9, A19, A20, A23, A53, A54, A55, A57, A58, A63 (A16 is at the 15th-digit
  boundary).
- Above the mode, from 1.05a to 1.45a, Excel is neither form (window A11:A20, A31:A40).
- POISSON.DIST's cdf far above the mean at k 100 and 200 is closer to the ratio form (window A129, A139, A140),
  while k 50 is not (A117, A118).
Many of these cells are visibly off (past the 15th significant digit). Read docs/research/excel-kernel-playbook.md first.

## Acceptance criteria
- [ ] A candidate model is scored against the cells above with a fixed hold-out (decision 0188 R4), and is adopted
      if it beats both forms; otherwise each rejected form is recorded in statistics.md with its scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign. Its target is that
every cell it owns is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet
then move to a follow-up task filed without the key and with `defer: phase-1b`.

## Out of scope
- Shapes below 20 (T-3219).
