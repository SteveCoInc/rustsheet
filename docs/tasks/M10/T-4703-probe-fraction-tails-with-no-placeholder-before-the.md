---
id: T-4703
title: "Probe fraction tails with no placeholder before the point and with a second point"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-4594]
---

## Goal
Oracle evidence for two fraction-tail shapes T-4594's point rule does not cover, recorded in number-formats.md, and
the engine matching it or a follow-up task filed.

## Context
- From the Excel-fidelity review of T-4594 (PR #3693, 2026-11-06). T-4594's `trailing_digits` rule
  (crates/rustsheet-engine/src/formatter/fraction.rs, number-formats.md "Placeholders after a fraction") was fitted on
  tails with placeholders on both sides of a `.`. Two shapes now take the new code path with no oracle data:
  - a tail with no placeholder before the point: `# ?/7 .0`, `# ?/7 .??`, `# ??/7 .#`;
  - a tail with a second `.`: `# ?/7 ?.?.?`, `# ?/7 0.0.0`.
- Use the `excel-probe` skill: a probe case under `eval/suites/probes/cases/` over values that spill digits
  (0.0123, 0.5, 1.75, 123.456), lacking zeros and nothing due, the same values T-4594's exploration grids used.

## Acceptance criteria
- [ ] A probe case with both shapes, oracle-filled (excel-graph).
- [ ] number-formats.md "Placeholders after a fraction" states what Excel does for each shape, with cells.
- [ ] The engine matches every cell, or the misses are listed in a new task and in the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the rule for the shapes T-4594 probed.
