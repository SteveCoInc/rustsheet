---
id: T-90007
title: "A date text with a space after each hyphen coerces to a date as Excel does"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A date text with a space after each hyphen (`9- Mar- 1999 `-style) coerces to a date under arithmetic as Excel does.

## Context
Owns: probes/date-text-spaced-hyphens (every visibly-off cell).
- S!B1, B3, B6: `A+0` is 36224 in Excel, B7 37970; ours #VALUE!. E1, E3, E6, E7 `ISNUMBER(A*1)` TRUE; ours FALSE. The case holds the variants that Excel refuses too; record the accepted forms in docs/excel-semantics before changing the date parser.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/date-text-spaced-hyphens matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
