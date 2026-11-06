---
id: T-2946
title: "CHAR lifts over an array argument"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`CHAR` over an array or range argument returns an array of characters, as Excel does, instead of `#N/IMPL!`:
`=LET(s,SEQUENCE(10),CONCAT(CHAR(48+s)))` is "123456789:".

## Context
- Found in T-2759: a first fill of probes/t2759-lunar-packed-* computed
  `CONCAT(IF(...,CHAR(80+g),CHAR(48+g)))` over a column array and Excel returned the packed string
  (excel-graph, 2026-10-27; the committed cases use MID instead), while the engine gives `#N/IMPL!`
  because `fn_char` (functions/text/char_code.rs) reads one number with `get_number`.
- CODE and UNICHAR beside it already lift through `lift_text`; reuse it. Per-element errors
  (0, 256, text, blank) should follow the scalar rules; check probes/text-argument-modes for an
  existing CHAR row before probing anew (excel-probe skill).

## Acceptance criteria
- [ ] A probe of CHAR over an array (including 0, 256, text and blank elements) passes, with no new known failures.
- [ ] A unit or eval case covers CHAR over an array with an out-of-range element.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The task's premise was stale: CHAR already lifts (T-3302, `lift_text`). Added probes/t2946-char-array (7 formulas, 18 cells, excel-graph filled, all pass with no engine change) covering 0, 256, text and blank elements and the packed-string formula, and recorded the per-element rules in text.md. Eval delta: 18 new passing cells, no regressions.
