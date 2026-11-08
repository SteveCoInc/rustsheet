---
id: T-5104
title: "General formatter: one helper for the E-form digit room, and a non-finite guard"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
General's E-form digit room (6 significant digits, 5 from exponent 99) is defined once in formatter/general.rs, and
`format_general` handles a non-finite value explicitly instead of producing "0E+00".

## Context
- Review MINORs on #4104 (T-4333). Design: the 6-or-5 rule is written twice, in `format_general_scientific`'s
  `digits_for` closure and in `scientific_room`. `scientific_room` takes its exponent from `log10().floor()`, while the
  formatter takes it from `significant_15`, so the two could disagree for a value that rounds up across 1E+99.
  Security: `format_general(NaN or ±inf)` returns "0E+00". No current caller passes a non-finite value.
- T-4333 left the code unchanged on purpose: the fresh sweep probes/t4466-general-full (240 of 240) tested the frozen rule.
- number-formats.md "General" holds the semantics.

## Acceptance criteria
- [ ] One helper gives the E-form digit count from the exponent `significant_15` yields, and both the choice and the
  formatter use it. The t4466-general-full and t4466-general-fixed-or-e cells all still pass.
- [ ] A non-finite value is guarded (unit test), and the behavior stays the same for every finite input.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to which form General picks.
