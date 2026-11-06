---
id: T-4062
title: "Probe a comma between placeholders in a scientific exponent (0.00E+0,0 and 0E+0,0)"
milestone: M10
lane: functions-text
status: done
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
- [x] A probe case covers the exponent forms; number-formats.md records the rule; the cells pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed the comma shapes: probes/t4062-comma-in-exponent (16 cells) and t4062-grouping-in-scientific
(44 cells; excel-graph 2026-11-06). A `,` between exponent placeholders is ignored after a mantissa
decimal point and `#VALUE!` without one (formatter/parser.rs `Token::Comma`); a grouping `,` in the
mantissa makes the engineering step at least 4 (formatter/format.rs). The probe also showed that
engineering mantissas round halves away from zero (`#,##0E+0` on 12.5 is "13E+0"), so
`round_engineering` now shares `significant_digits` with `round_scientific` instead of
`to_precision` (half-even on exact ties). number-formats.md gains "Commas in scientific formats".
Both cases pass in full; 0 regressions.
