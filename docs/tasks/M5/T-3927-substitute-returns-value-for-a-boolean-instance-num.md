---
id: T-3927
title: "SUBSTITUTE returns #VALUE! for a boolean instance_num (gen-m5 arg-pairs, T-3510)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
campaign: semantics
size: S
tier: sonnet
depends_on: []
---

## Goal
`SUBSTITUTE(text, old_text, new_text, instance_num)` returns `#VALUE!` when instance_num is a boolean, as Excel does.

## Context
- Evidence: gen-m5 arg-pairs batch of T-3510 (excel-graph oracle): all 23 failing SUBSTITUTE cells have a boolean
  `TRUE` as the fourth argument and Excel gives `#VALUE!`; the engine coerces TRUE to 1 and returns the text with
  the first instance replaced. Cases (case file stem and A1 names, decision 0159):
  gen-m5/5136e04c4ce978c3 A79, A133, A217, A289, A337, A451, A517, A559; gen-m5/5614fae2486f6dd5 A67, A109, A157,
  A223, A277, A385, A451; gen-m5/edae081ad96b1d56 A259, A313, A373, A421, A475, A583; gen-m5/f01da2cf9f4f2105 A109, A175.
- Code: `fn_substitute` (crates/rustsheet-engine/src/functions/text/common.rs) passes `calc_result_to_number` of the
  fourth argument to `substitute_element` (functions/text/util.rs).
- Whether a boolean from a cell reference behaves like the literal, and FALSE, are unprobed here: probe first
  (`excel-probe` skill) before implementing.

## Acceptance criteria
- [ ] The 23 cells above pass and leave eval/baseline/gen-m5.txt; 0 regressions.
- [ ] docs/excel-semantics/text.md entry "SUBSTITUTE boolean instance_num" says "Implemented in" this task.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Wrong error: the engine returns text where Excel gives `#VALUE!`.

## Out of scope
- SUBSTITUTE's other argument kinds (all other pairs of the T-3510 batch pass).
