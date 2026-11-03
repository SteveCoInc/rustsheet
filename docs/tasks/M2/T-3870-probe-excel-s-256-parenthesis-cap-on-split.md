---
id: T-3870
title: "Probe Excel's 256-parenthesis cap on split runs (space, operand, call between groups) and re-probe the typed refusals with ISBLANK/LEN"
milestone: M2
lane: research
status: done
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
- [x] Probe cases at 256 and 257 open groups for each split shape, stored and typed, filled from excel-graph; syntax.md
      states Excel's rule; any disagreement gets an owner task.
- [x] The restored typed-refusal probes check `ISBLANK(A1)` too, refilled, and pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- 16 new probe cases, probes/t3870-{space,operand}-{256,257} and t3870-{call,call-mid}-{255,256,257}, stored and
  typed, filled from excel-graph 2026-11-03 (A1 the formula, B1 `ISFORMULA(A1)`, D1 `ISBLANK(A1)`). Excel counts
  groups split by a space, an operand or a call together against the 256 cap, and a function call's own `(` is not
  a group: 256 groups load (inside `ABS(` too, 257 `(` open), 257 are refused. The four refused stored cases made
  Excel for the web refuse the workbook (createSession 501 `FileCorruptTryRepair`) and were dropped, as T-3172's
  were; the typed 257s are PATCH 400 and are kept. syntax.md ("Formula nesting depth cap") states the rule;
  `MAX_PARENTHESES`'s doc no longer calls split runs unprobed.
- The 23 restored typed refusals (t3097-*-66-typed, t3172-*-typed, t3266-*-typed; the task said 19) now check
  `ISBLANK(A1)` in D1, refilled: TRUE on all 23, so Excel leaves A1 blank, not text.
- Ours already matches every cell: no engine change, no disagreement, so no owner task.
- Eval delta: probes known failures 4995 -> 4995; 16 new cases and 23 cases with one more checked cell, all
  passing; no new known-failure line.
