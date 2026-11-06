---
id: T-4625
title: "Probe GAMMA.DIST's overflowing x / beta at shapes other than 1"
milestone: M0
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence says what GAMMA.DIST (both forms) returns when x / beta overflows at a shape other than 1, and the
Gate A definition's "x / beta overflows" `#NUM!` rule covers only the shapes that evidence supports.

## Context
- `_gamma_dist_domain` (tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py) returns `#NUM!` when x / beta
  overflows, at every shape. The only evidence is probes/incomplete-gamma-cap E6 and E18, both at shape 1
  (statistics.md T-2738 entry, as narrowed by T-4395). Other shapes are an unevidenced error claim.
- Raised by T-4395's excel-fidelity review.

## Acceptance criteria
- [x] A probe case with oracle values for `GAMMA.DIST(1E300,a,1E-10,TRUE)` and `FALSE` at shapes such as 0.5, 2, 10
      and 1E5.
- [x] The statistics.md T-2738 entry states the rule for the probed shapes; the definition raises `#NUM!` only where
      the evidence shows it and `NoValueError` elsewhere; `rustsheet-accuracy verify GAMMA.DIST` and `GAMMADIST` give 0
      differences.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine; file a functions task if the engine disagrees with the new cells.

## Outcome
- New probe probes/t4625-gamma-dist-ratio-overflow (excel-graph 2026-11-06, 62 cells). An overflowing x / beta is
  `#NUM!` for both forms, GAMMA.DIST and GAMMADIST, at shapes 0.01, 0.5, 2, 10, 1E5, 1E10 and 1E300 (rows 1-8). With
  the Gate A cells GAMMADIST/s1 A182 (shape 3.7E-188), A118 and GAMMA.DIST/s1 A141 the evidenced range is 3.7E-188 to
  1E300. A typed `1E308` literal makes Excel for the web refuse the workbook, so row 8 divides 1E300 by 5E-9.
- `_gamma_dist_domain` now raises `#NUM!` for an overflowing ratio only in that shape range (`OVERFLOW_SHAPES`) and
  gives no value outside it.
- Found in passing, unexplained: at the finite x / beta = 1E308, shapes 1.5 and 2 are `#NUM!` (both forms; B9, F9,
  B10, B11), while 0.5, 1, 2.5, 2.9, 3, 10 and 1E5 are numbers, as is shape 2 from x / beta 1E200 down. The definition
  gives no value above 1E200 between shapes 1 and 2.5 (`HUGE_RATIO_*`). The engine gives 1 / 0 at those four cells;
  T-4635 owns them. Recorded in statistics.md (the T-2738 entry).
- `rustsheet-accuracy verify GAMMA.DIST` and `GAMMADIST`: 0 differences each.
- Eval delta: +62 probe cells, 58 pass, 4 known-off (B9, F9, B10, B11; T-4635) added to eval/baseline/probes.txt. No
  engine change.
