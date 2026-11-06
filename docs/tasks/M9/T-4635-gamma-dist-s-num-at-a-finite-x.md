---
id: T-4635
title: "GAMMA.DIST's #NUM! at a finite x / beta of 1E308 at shapes 1.5 and 2: bracket it and match"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine gives Excel's `#NUM!` where GAMMA.DIST (both forms) is `#NUM!` at a huge finite x / beta, by a rule that
oracle evidence brackets, and the Gate A definition gives those points a value or a domain error by that rule.

## Context
- T-4625 found that `GAMMA.DIST(1E300,a,1E-8,..)` (x / beta = 1E308, finite) is `#NUM!` for the cdf and the density
  at shapes 1.5 and 2, and a number (1 / 0) at shapes 0.5, 1, 2.5, 2.9, 3, 10 and 1E5; at shape 2 it is a number
  from x / beta = 1E200 down. Neither "y^a overflows" (2.5 is fine) nor "a * y overflows" (1.5 fails) fits.
- Off cells (the engine gives 1 and 0): probes/t4625-gamma-dist-ratio-overflow B9, F9, B10, B11.
- statistics.md, "GAMMA.DIST's density is #NUM! at a subnormal x / beta (T-2738)" (the T-4625 paragraph); the Gate A
  definition gives no value there (tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py `HUGE_RATIO_*`).
- Engine: crates/rustsheet-engine/src/functions/statistical/gamma.rs `gamma_dist_of`.

## Acceptance criteria
- [ ] A probe brackets the region: shapes between 1 and 2.5 (e.g. 1.1, 1.25, 1.75, 2.25, 2.4) and x / beta between
      1E200 and 1.7E308 at shape 2 and 1.5, both forms; the rule is recorded in statistics.md.
- [ ] The engine matches every cell of probes/t4625-gamma-dist-ratio-overflow and the new probe (B9, F9, B10, B11
      included), and the Gate A definition's `HUGE_RATIO_*` no-value region narrows to what the rule cannot cover.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The overflowing x / beta rule (T-4625; the engine already matches it).
