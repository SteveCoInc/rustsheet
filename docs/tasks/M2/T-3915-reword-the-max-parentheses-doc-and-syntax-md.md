---
id: T-3915
title: "Reword the MAX_PARENTHESES doc and its syntax.md notes (T-3870 review MINORs)"
milestone: M2
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The `MAX_PARENTHESES` doc comment and the two T-3870 sentences in docs/excel-semantics/syntax.md read correctly. Only the
wording changes, not what they claim.

## Context
- MINOR findings of all three reviewers on PR #2955 (T-3870). They were deferred because every reviewer approved.
- crates/rustsheet-engine/src/expressions/parser/depth.rs, the doc of `MAX_PARENTHESES`: "and so do levels of `-(`" now
  follows the new call clause, so it reads as if `-(` levels were also "not a group". Make the call-parenthesis point
  its own sentence, after the `-(` clause, and keep lines within the usual width.
- docs/excel-semantics/syntax.md, "Formula nesting depth cap": the parenthetical "(T-3870 added `ISBLANK(A1)` ... TRUE
  on all 23 ...)" sits inside the T-3689 sentence, before "so T-3689 restored them". That makes it read as the reason
  for T-3689's restore. Move it to its own sentence after the T-3689 one.
- Salvaged from PR #2964 (closed as a duplicate of #2959, which landed T-3883). That PR's T-3883 MINORs
  concerned its own code, which never landed, so they are dropped.

## Acceptance criteria
- [ ] Each place above is reworded or moved, and the facts and the cited cases are unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to the parser or to a probe case.
