---
id: T-4891
title: "Probe GAMMA.INV where Excel's huge-shape start times the scale overflows: #NUM! though the true quantile fits?"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3364]
---

## Goal
Oracle evidence settles what Excel returns where its huge-shape Wilson-Hilferty start times the scale overflows but the
true quantile still fits in a double, and the engine matches it.

## Context
- T-3364 (decision 0337) moved GAMMA.INV's closed-form branch to the math. Its excel-fidelity review found a band
  where Excel's value (its start, about 1.000266 times the true quantile) times the scale overflows: shape 1E300,
  scale about 1.7972E8 to 1.7977E8. There `huge_shape_quantile` (statistical/incomplete_gamma/huge_shape.rs) returns
  NaN (`#NUM!`), as main did before T-3364. Decision 0188 R3 never trades an error for a number. The result is
  reasoned from Excel's model, not oracle-confirmed (UNVERIFIED).
- Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case holds `GAMMA.INV(0.5,1E300,1.7975E8)`, a neighbour just below the overflow edge (for example scale
      1.797E8) and one inside the band for GAMMAINV and CHISQ.INV.RT, all oracle-filled.
- [ ] The engine matches every cell's type (`#NUM!` or a number), and the statistics.md entry "GAMMA.INV at a huge
      shape, closed-form branch" records the finding.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The branch's digits (R3, T-3364).
