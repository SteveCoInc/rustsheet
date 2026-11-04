---
id: T-4062
title: "Probe a comma between placeholders in a scientific exponent (0.00E+0,0 and 0E+0,0)"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT with a `,` between digit placeholders in a scientific exponent (`0.00E+0,0`, `0E+0,0`, `#,##0.0E+0,0`) prints what Excel prints.

## Context
T-3598's Excel-fidelity review (MINOR): formatter/parser.rs `Token::Comma` now ignores a `,` between placeholders after the decimal point (probes/t3598-comma-after-decimal-point; number-formats.md "Commas after the decimal point"). That branch also catches a `,` in the exponent after a decimal point, which before T-3598 turned on grouping. Neither behaviour has Excel evidence. Probe the exponent forms with and without a decimal point, then match them.

## Acceptance criteria
- [ ] A probe case covers the exponent forms; number-formats.md records the rule; the cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
