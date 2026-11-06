---
id: T-4595
title: "A literal right after a number section's decimal point prints after the point (`0.|0` on 1234567.891 is \"1234567.|9\")"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A literal written right after the decimal point of a number section prints after the point, as Excel shows it.

## Context
- Ours prints the point after the literal: `TEXT(1234567.891,"0.|0")` and `"0.,|0"` are "1234567|.9" here.
- Excel: `0.,|0` on 1234567.891 is "1234567.|9", on -0.5 "-0.|5", on 0 "0.|0" (probes/t4515-fresh-sweep A15:D15,
  excel-graph 2026-11-06; T-4515). The `,` there is inert (number-formats.md "Commas after the decimal point"), so the
  same holds for `0.|0`, which no probe has yet.
- Code: the number branch of `format_number` (crates/rustsheet-engine/src/formatter/format.rs), where the `Period`
  token prints; find why a literal between the point and the first decimal placeholder moves ahead of it.

## Owned cases
- probes/t4515-fresh-sweep A15:D15

## Acceptance criteria
- [ ] probes/t4515-fresh-sweep A15:D15 pass, with no regression in any suite.
- [ ] A probe of `0.|0`, `0."x"0`, `#.\-#` and `0.|` (three values) is committed with excel-graph values and passes.
- [ ] number-formats.md records the rule with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Commas after the decimal point (T-4515).
