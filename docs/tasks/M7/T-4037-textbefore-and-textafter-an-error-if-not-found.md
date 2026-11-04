---
id: T-4037
title: "TEXTBEFORE and TEXTAFTER: an error match_end beats the #VALUE! of an out-of-range numeric match_mode (gen-m7)"
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
TEXTBEFORE and TEXTAFTER return an error-valued `match_end` (5th argument) where Excel does, so the 11 gen-m7 cells below match.

## Context
Found by the T-3513 arg-pairs batch (excel-graph). Signature: text, delimiter, instance_num, match_mode, match_end,
if_not_found. In each failing cell match_end is an error value (a literal such as `#N/A` or `Data!G9` = `#DIV/0!`) and
match_mode is a number outside {0, 1}: 2, 7 (`Data!G1`), -1, -2.5 (`Data!G3`), "12" (`Data!G5`) or "2.5". Excel returns
the match_end error; the engine returns `#VALUE!` for the invalid match_mode. The boundary is already in the batch: a
non-numeric match_mode ("x" in Data!G6, "", blank G7) still gives `#VALUE!` beside an error match_end (9f9357dbf7e17803
A343, A346, A409, A412; b2139ff37d606cb4 A373, A376, A439, A442), so the rule looks like "an error match_end wins over
the `#VALUE!` of an out-of-range numeric match_mode, not over a match_mode that fails to coerce". Instance_num 0 is not
the cause (it passes elsewhere). It is an error-ordering question inside `around_options` in the TEXTBEFORE/TEXTAFTER
module of `crates/rustsheet-engine`. Use the `excel-probe` skill to fix the boundary first (the lock lines come out with
`--lower-visible-lock`). docs/excel-semantics/text.md records the observation (not implemented).
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
