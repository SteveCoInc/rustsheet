---
id: T-3870
title: "Probe Excel's 256-parenthesis cap on split runs (space, operand, call between groups) and re-probe the typed refusals with ISBLANK/LEN"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for how Excel counts the 256-parenthesis cap over split runs, and typed-refusal probes that tell a blank
A1 from text.

## Context
- Follow-ups of the T-3689 review (excel-fidelity reviewer, MINOR). The parser counts groups split into several runs
  together (`( ((...)))` with a space, `(1+((...)))` with an operand, `ABS((...))` with a call). Since T-3689 it refuses
  more than 256 open groups as Excel's syntax (`TOO_MANY_PARENTHESES_MESSAGE`). syntax.md says that only `-(` is probed
  (t3266-nested-paren-256/-257). The space, operand and call splits are unprobed.
- The 19 typed-refusal probes T-3689 restored (t3097-nesting-*-66-typed, t3172-*-typed, t3266-*-typed) score only
  `ISFORMULA(A1)` FALSE, which text in A1 would also give. Adding `ISBLANK(A1)` (or `LEN(A1)`) and refilling would pin
  the blank cell Excel leaves.

## Acceptance criteria
- [ ] Probe cases at 256 and 257 open groups for each split shape, stored and typed, filled from excel-graph; syntax.md
      states Excel's rule; any disagreement gets an owner task.
- [ ] The restored typed-refusal probes check `ISBLANK(A1)` too, refilled, and pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
