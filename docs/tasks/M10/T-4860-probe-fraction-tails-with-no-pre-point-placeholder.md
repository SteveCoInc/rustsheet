---
id: T-4860
title: "Probe fraction tails with no pre-point placeholder under a lacking denominator"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4703]
---

## Goal
Oracle evidence for fraction tails with no digit placeholder before the point when the picked denominator lacks digits
against the code (`# ?/16 .0` on 0.5), and the engine matching it.

## Context
- From the Excel-fidelity review of T-4703 (PR #3891). T-4703 showed spilled digits print right before a point with no
  placeholder ahead of it (number-formats.md "Placeholders after a fraction", fraction.rs `trailing_digits`). Lacking
  digits (picked denominator shorter than the code) were not probed there: every code was `?/7`.
- Probe `# ?/16 .0`, `# ?/100 .0?`, `# ?/16|.0`, `# ?/100 .##` on 0.5, 0.25, 0.3, 1.75 (excel-probe skill).

## Acceptance criteria
- [ ] A probe case over those shapes, oracle-filled (excel-graph).
- [ ] number-formats.md states where lacking zeros print when no placeholder precedes the point.
- [ ] The engine matches every cell, or the misses are listed in a new task and in the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The shapes T-4594 and T-4703 probed.
