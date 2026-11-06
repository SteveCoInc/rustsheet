---
id: T-4578
title: "PROB without a per-probability range check and with Excel's sum tolerance; CHISQ.TEST and Z.TEST overflow errors (T-4098 probe)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PROB, CHISQ.TEST and Z.TEST give Excel's results on the cells T-4098's probe and tables found off: PROB checks only the
sum of the probabilities, with Excel's tolerance; CHISQ.TEST is `#NUM!` for an overflowing squared difference; Z.TEST
answers Excel's `#NUM!` and 0.5 where z overflows with sigma given (probe A48, A49, one cell each; with sigma omitted
it is unprobed and out of scope).

## Context
- statistics.md "CHISQ.TEST, Z.TEST and PROB: Gate A domains (T-4098)" states the rules and the evidence.
- PROB (`crates/rustsheet-engine/src/functions/statistical/prob.rs`): the engine is `#NUM!` for any probability outside
  [0, 1], where Excel checks only the sum (`PROB({1,2},{1.5,-0.5},1)` is 1.5), and accepts a sum within 1E-14 of 1,
  where Excel refuses 1E-15 below and 1E-14 above but accepts 1E-15 above. Cells: probes/t4098-tests-prob-domains A3,
  A4, A5, A32, A33; accuracy/PROB/s1 A113, A125 (known-failures.tsv lines owned here).
- CHISQ.TEST (`statistical/chisq.rs`): a squared difference that overflows gives p 0 in the engine, `#NUM!` in Excel:
  probes/t4098-tests-prob-domains A44, A45.
- Z.TEST (`statistical/z_test.rs`): with sigma 1E-300, x = -1E+300 is `#NUM!` in Excel (engine 0) and x = 1E+300 is 0.5
  (engine 1): probes/t4098-tests-prob-domains A48, A49.
- The exact cut of PROB's tolerance is not pinned: the cells fit "the double sum cut to 15 significant digits is 1".
  A probe between the probed amounts (for example 3E-15 and 6E-15 above 1, 2E-16 and 6E-16 below) can settle it first.

## Acceptance criteria
- [ ] probes/t4098-tests-prob-domains A3, A4, A5, A32, A33, A44, A45, A48 and A49 pass.
- [ ] accuracy/PROB/s1 A113 and A125 pass, and their known-failures.tsv lines are removed.
- [ ] The semantics entry's "Ours" line is updated; `make check` green; reviewers approve.

## Out of scope
- CHISQ.TEST's df for equal counts of different shapes (T-2650).
