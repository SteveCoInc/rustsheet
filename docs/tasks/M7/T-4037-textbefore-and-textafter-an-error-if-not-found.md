---
id: T-4037
title: "TEXTBEFORE and TEXTAFTER: an error if_not_found is returned in place of the #VALUE! of an invalid or failing call (gen-m7)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
TEXTBEFORE and TEXTAFTER return an error-valued `if_not_found` where Excel does, so the 11 gen-m7 cells below match.

## Context
Found by the T-3513 arg-pairs batch (excel-graph). In each cell the 5th argument is an error value (literal or a
`Data!G9` cell) and Excel returns that error where the engine returns `#VALUE!`. The calls are ones whose other
arguments are invalid or find no match (a match_mode of -1 or 2, instance_num 0, a non-number instance or match_mode
cell holding an error, a "2.5" text instance), so the rule looks like "a call that would fail returns if_not_found,
whatever the cause", but the boundary (which invalid arguments still give `#VALUE!`) is not probed. Use the
`excel-probe` skill first. Code: the TEXTBEFORE/TEXTAFTER module in `crates/rustsheet-engine`.
Cells (case stem and A1 names, decision 0159):
- gen-m7/9f9357dbf7e17803: A85, A88, A157, A160, A286 (TEXTAFTER)
- gen-m7/b2139ff37d606cb4: A115, A118, A199, A202, A313, A316 (TEXTBEFORE)

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong error (none is a last-digit difference), so the task is in
the campaign. Its target is that every cell it owns has Excel's type, error and text.

## Acceptance criteria
- [ ] A probe case with oracle values fixes the rule and its boundary; docs/excel-semantics/text.md records it.
- [ ] The 11 cells above pass and leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other TEXTBEFORE/TEXTAFTER arguments and TEXTSPLIT.
