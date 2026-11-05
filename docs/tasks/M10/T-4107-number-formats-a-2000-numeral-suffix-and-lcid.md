---
id: T-4107
title: "Number formats: a ,2000 numeral suffix (and LCID byte 32) prints fullwidth digits in Excel"
milestone: M10
lane: functions-text
status: todo
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(123,"[$-ar-SA,2000]0")` gives Excel's fullwidth `１２３`, and the engine knows which numeral byte, if any, the
`[$-N000409]` LCID form maps to fullwidth beyond 29.

## Context
probes/t3891-numeral-tag-render A28 (excel-graph 2026-11-04): `[$-ar-SA,2000]0` on 123 is `１２３`, while
T-3891's `numeral_suffix` reads `,2000` as byte 0x20 (32), which `Numerals::from_byte`
(formatter/numerals.rs) maps to ASCII, as the LCID form `[$-20000409]` does by T-1925's evidence.
Every other probed suffix (`,200` to `,1200`, `,0`) is byte = value >> 8. Probe first (excel-probe):
`[$-20000409]0`, `[$-,2000]0`, `[$-,1D00]0`, `[$-,1F00]0`, `[$-,2100]0`, `[$-,FF00]0`, `[$-,201]0`,
`[$-,20]0` and a date field, then fix `locale_tag::numerals` (TODO(T-4107) there).

## Acceptance criteria
- [ ] The probe's cells and probes/t3891-numeral-tag-render A28 pass; the rule is in
      docs/excel-semantics/number-formats.md "Numeral-system digits".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed script-digit input (T-3891).
