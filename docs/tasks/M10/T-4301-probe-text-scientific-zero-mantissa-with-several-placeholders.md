---
id: T-4301
title: "Probe TEXT scientific zero mantissa with several ? placeholders and multi-section E+ with no exponent digit"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for a TEXT scientific zero mantissa with several `?` integer placeholders, and for `E+` with no exponent digit in a multi-section format.

## Context
T-2230 (PR 3296) made a `?` integer placeholder print 0 on a zero mantissa (`?.?E+0` on 0 is "0. E+0"; probes/text-scientific-edges A41) and `E+`/`E-` with no digit `#VALUE!`. Multi-`?` forms (`??.?E+0`, `???E+0`, `?#.?E+0`) and multi-section forms (`0;0E+`, applied to negative values) are unprobed; the code treats every non-`0` placeholder alike (formatter/format.rs).

## Acceptance criteria
- [ ] A probe case with the forms above on 0, 1.5 and -1.5, oracle-filled from excel-graph; the engine matches or mismatches are filed.
- [ ] Findings added to the number-formats.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
