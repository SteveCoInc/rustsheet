---
id: T-4595
title: "A literal right after a number section's decimal point prints after the point (`0.|0` on 1234567.891 is \"1234567.|9\")"
milestone: M10
lane: functions-text
status: done
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
- [x] probes/t4515-fresh-sweep A15:D15 pass, with no regression in any suite.
- [x] A probe of `0.|0`, `0."x"0`, `#.\-#` and `0.|` (three values) is committed with excel-graph values and passes.
- [x] number-formats.md records the rule with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Commas after the decimal point (T-4515).

## Outcome
- The number branch of `format_number` (formatter/format.rs) prints the decimal point where the `Period` token
  stands instead of deferring it to the first decimal digit, so a literal right after the point follows it
  (`0.,|0` and `0.|0` on 1234567.891 are "1234567.|9"). The deferral changed nothing else: every decimal-digit
  branch printed the deferred point first, so the `needs_period` state is gone.
- Probe probes/t4595-literal-after-point (`0.|0`, `0."x"0`, `#.\-#`, `0.|` on 1234567.891, -0.5, 0): predictions
  committed before the excel-graph fill (2026-11-06); all 12 cells matched them and pass.
- Unit test `literal_right_after_the_point_prints_after_it` (formatter/test/test_digit_edges.rs).
- number-formats.md: new "A literal right after the decimal point (T-4595)" entry; the T-4515 note points to it.
- Eval delta: probes known failures 7308 -> 7304 (t4515-fresh-sweep A15:D15 fixed), new case 12/12 passing, no
  regression in any suite.
