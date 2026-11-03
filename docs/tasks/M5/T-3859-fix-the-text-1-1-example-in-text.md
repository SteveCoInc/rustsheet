---
id: T-3859
title: "Fix the TEXT(-1,-1) example in text.md's format_text entry: cite A259 TEXT(-0.5,-1)"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3809]
---

## Goal
docs/excel-semantics/text.md "TEXT's format_text of every argument class (T-3809)" cites only cells the batch has.

## Context
The T-3809 Excel-fidelity review (PR #2862, MINOR) found that the entry's Rule gives `TEXT(-1,-1)` is "--1", but
gen-m5/0428ca72d3724bea has no such cell. The cell is Calc!A259, `TEXT(Data!A1:B2,-1)` = "--1", whose top-left value is
-0.5. The phrase "a literal `-` before a digit" is also misleading: the `1` is a literal character, not a digit
placeholder, so the value itself is never shown, only its minus (A49 `TEXT("2.5",2)` = "2", A133 `TEXT(TRUE,2)` = "TRUE").
The fix could not go in PR #2862 after approval (work-loop step 5: a MINOR never starts a push).

## Acceptance criteria
- [ ] The example reads `TEXT(-0.5,-1)` is "--1", and the code is described as the literals `-` and `1` with no digit
      placeholder, so only the value's minus is added.
- [ ] The entry's Evidence line cites A259.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any other text.md entry.
