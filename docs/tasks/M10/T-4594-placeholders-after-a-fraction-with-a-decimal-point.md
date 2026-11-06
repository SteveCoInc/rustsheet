---
id: T-4594
title: "Placeholders after a fraction with a decimal point among them: one rule for the digits on each side of the tail's point"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Digit placeholders after a fraction's denominator with a `.` among them print as Excel does, by one rule for the
placeholders on each side of that point, extending the T-4515 model.

## Context
- The T-4515 model (number-formats.md "Placeholders after a fraction") fits every tail without a `.` (1,300+ cells and
  its hold-out) but not a tail with one. Before the point a `?` seems to take every spilled digit, not one: `# ?/7 ?.?`
  on 0.0123 is " 123/7 0000. " (probes/t4515-fraction-tail-holdout E6). After the point the owed zeros are not paid and
  `#` prints differently: `# ?/7 ?.0` on 0.0123 is " 123/7 0000.0" (one zero, not two), `# ?/1000|?.#` on 0.0123
  " 123/1000|0." but on 0.123456789 " 10/10  |0.0", `# ?/40|?.0#` on 0.0123 " 123/40|000.00"
  (probes/t4515-fresh-sweep rows 1-8, excel-graph 2026-11-06).
- A "take all before the point, owe one" rule was tried in T-4515 and falsified by that sweep. Treat the tail as a
  small number section: the spilled digits as its integer, the lacking digits and owed zeros as its decimals.
  Hold-out: a fresh probe written after the model is frozen.
- Code: `trailing_digits` in crates/rustsheet-engine/src/formatter/fraction.rs.

## Owned cases
- probes/t4515-fraction-tail-holdout E6
- probes/t4515-fresh-sweep A1:G3, A5:G6, A8:G8

## Acceptance criteria
- [ ] The owned cells pass, and every T-4515 case still passes, with no regression in any suite.
- [ ] A hold-out probe made after the model froze passes.
- [ ] number-formats.md "Placeholders after a fraction" states the point rule with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Commas after the decimal point; a literal right after a number section's point (T-4595).
