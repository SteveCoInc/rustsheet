---
id: T-4284
title: "Probe Lotus dot ranges with whitespace after the dots on a cell or a line feed (. B2, A1.LF B2) and pin the lexer's fallback"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4271]
---

## Goal
The two readings of whitespace after the dots that T-4271 (#3270) implemented without evidence have excel-graph
evidence, and a lexer test pins the fallback when no second cell follows.

## Context
- T-4271 added `Lexer::dotted_range_after_cell_name` (crates/rustsheet-engine/src/expressions/lexer/ranges.rs). It
  fires on any whitespace after the dots, so two unprobed forms now read as ranges as well (excel-fidelity review MINOR
  on #3270):
  - a line feed after the dots, `A1.`&LF&`B2` (only a line feed before the dot is probed:
    probes/t4271-lotus-dot-one-side-lf-before);
  - a `$` cell before the dots, `$A$1. B2` and `A$1.. B2` (only cells without `$` are probed).
- docs/excel-semantics/references.md, the Lotus-range bullet (T-4271 part), records what is probed.
- The fallback (no second cell after the dots: `A1. 5`, `A1. )`) restores the lexer position. Excel refuses
  `A1. 5` at load (`FileCorruptTryRepair`), so no oracle case covers it, and no lexer test checks it either.

## Acceptance criteria
- [ ] Probe cases, one form per case (excel-probe skill; split as `t4271-lotus-dot-one-side-*` did), for `A1.`&LF&`B2`,
      `$A$1. B2` and `A$1.. B2`: SUM, ROWS, ISREF and FORMULATEXT. Each is filled on excel-graph, or named as refused
      with the Graph error. The engine matches what Excel does.
- [ ] A unit test in crates/rustsheet-engine/src/test/test_lotus_dot_ranges.rs: `=SUM(A1. 5)` and `=SUM(A1. )` give
      the same parse error as before T-4271 (`#ERROR!`), and `=A1. +1` keeps its old reading.
- [ ] references.md updated with the probed readings.
- [ ] `make check` green; reviewers approve.

## Out of scope
- R1C1-mode dotted forms.
