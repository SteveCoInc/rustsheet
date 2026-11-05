---
id: T-4295
title: "TEXT \"[s] m\" shows the seconds field and 26640 (not 6138 42): find the rule"
milestone: M10
lane: functions-text
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`TEXT(1.7/24+0.3/1440,"[s] m")` matches Excel's "18 26640" in both date systems, with the rule documented.

## Context
Found by T-2135. probes/t2135-fractional-second-edges-1900 S!A39 and probes/t2135-fractional-second-edges-1904 S!A39: the engine gives "6138 42". `m [m]` already gives "1 26640", so 26640 is likely the same quantity; the rule for `[s]` printing 18 and for 26640 is unknown and needs more probes (other values, `[s] mm`, `[s]:m`). See number-formats.md "Elapsed-time tokens".

## Acceptance criteria
- [ ] Probe the rule (excel-probe skill), record it in number-formats.md, and make both A39 cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
