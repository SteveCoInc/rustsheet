---
id: T-4286
title: "Z.TEST: a lone cell reached through INDEX or IF is #N/A when it holds text or a boolean (INDEX(A1:A5,1), IF(TRUE,A1))"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Z.TEST and ZTEST treat a single cell reached through INDEX or IF like a plain cell reference: text, a boolean or an empty cell is `#N/A` (no number in the data), as Excel does.

## Context
probes/t2775-ztest-single-cell-routes (excel-graph 2026-11-05): with A1 text and A2 TRUE, `Z.TEST(INDEX(A1:A5,1),1)` is `#N/A` (ours `#VALUE!`), `Z.TEST(INDEX(A1:A5,2),1)` `#N/A` (ours `#DIV/0!`, the boolean read as a direct number), `Z.TEST(IF(TRUE,A1),1)` `#N/A` (ours `#VALUE!`). OFFSET, parentheses and a plain reference already pass. `fn_z_test` (z_test.rs) tells a reference only by the argument node being a `ReferenceKind`; INDEX and IF evaluate a single cell to its value, so the reference origin is lost. Needs a way to see that the argument was a reference (design question). Rule: docs/excel-semantics/statistics.md, T-2775 note.

Cells:
- probes/t2775-ztest-single-cell-routes.case.json (3): Sheet1!B3, B4, B13

## Acceptance criteria
- [ ] probes/t2775-ztest-single-cell-routes B3, B4 and B13 pass (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
