---
id: T-3849
title: "Probe text scalar against an error data cell in other forms (typed text, computed text, one-cell range, name, INDEX)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Establish with the excel-probe skill whether a non-numeric text scalar beats an error data argument in forms T-3788 leaves alone, and make the engine match: a typed text scalar (`SMALL(Data!G9,"x")`), a computed text scalar, a one-cell range as the data (`Data!G9:G9`), and a name, INDEX, OFFSET, IF or CHOOSE that picks the error cell.

## Context
T-3788 (sorted_data.rs `lift_over_sorted`) applies the rule only when both the data and the scalar are cell references; all 165 oracle cells use that form. Other forms keep the data's error with no evidence either way. Record the results in statistics.md "A text scalar argument is #VALUE! before an error cell as the data".

## Acceptance criteria
- [ ] Probe cases for each form have Excel values; statistics.md records them.
- [ ] The engine matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions.
