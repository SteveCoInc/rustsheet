---
id: T-4625
title: "Probe GAMMA.DIST's overflowing x / beta at shapes other than 1"
milestone: M0
lane: research
status: todo
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
- [ ] A probe case with oracle values for `GAMMA.DIST(1E300,a,1E-10,TRUE)` and `FALSE` at shapes such as 0.5, 2, 10
      and 1E5.
- [ ] The statistics.md T-2738 entry states the rule for the probed shapes; the definition raises `#NUM!` only where
      the evidence shows it and `NoValueError` elsewhere; `rustsheet-accuracy verify GAMMA.DIST` and `GAMMADIST` give 0
      differences.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine; file a functions task if the engine disagrees with the new cells.
