---
id: T-4659
title: "T-4578 review follow-ups: PROB's probed bands in statistics.md, Z.TEST's mean-minus-x overflow, the 15-digit rule in the Gate A PROB def"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The three MINOR findings of T-4578's review (PR #3709) are closed: statistics.md states PROB's probed bands exactly,
Z.TEST's other overflow source is probed or marked UNVERIFIED, and Gate A's PROB definition encodes the evidenced
15-digit rule.

## Context
- statistics.md "CHISQ.TEST, Z.TEST and PROB: Gate A domains (T-4098)", PROB tolerance bullet: the range summary says
  numbers for sums "2.2E-16 to 4.0E-15 above" 1 and `#NUM!` "6.0E-15 to 1E-14 above", "6.7E-16 to 1E-15 below". The
  probes/t4578-prob-sum-tolerance cells say: numbers from 1.1E-15 (A1) to 4.88E-15 (A17) above and 2.2E-16 to 4.44E-16
  below; `#NUM!` from 5.11E-15 (A5) to 1E-14 above and 5.55E-16 (A11, A12) to 1E-15 below (Excel-fidelity review).
  Also rewrap the long "The table adds sums 7.3E-9..." line (design review).
- Z.TEST (crates/rustsheet-engine/src/functions/statistical/z_test.rs, `z_test_p`): with sigma given, an infinite z is
  `#NUM!` when positive and 0.5 when negative. The evidence (t4098-tests-prob-domains A48, A49) has z overflow in the
  quotient by sigma = 1E-300. The code also covers an infinite z from `mean - x` overflowing, e.g.
  `Z.TEST({1E+308},-1E+308,1)` (was Phi(-inf) = 0, now `#NUM!`), which no cell tests. Probe it (excel-probe skill,
  predictions first), or mark it UNVERIFIED in statistics.md.
- Gate A: tools/src/rustsheet_tools/accuracy/defs/significance.py's PROB definition gives no math value to sums
  between the old probed bands and generates `#NUM!` sums only at 1E-14 above and 1E-15 below. Encode the evidenced rule
  (the double sum cut to 15 significant digits is 1; `decimal_round::cut_to_15_digits` in the engine) so those sums get
  a value, then regenerate the PROB table's math sidecar (`rustsheet-accuracy math --function PROB`).

## Acceptance criteria
- [ ] statistics.md's PROB bands match the t4578 cells, and the long line is rewrapped.
- [ ] Z.TEST's `mean - x` overflow is probed and passes, or is marked UNVERIFIED.
- [ ] defs/significance.py encodes the 15-digit rule; `rustsheet-eval accuracy` reports 0 failing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PROB's unprobed points (an exact 16th-digit tie, summation order for three or more probabilities).
