---
id: T-4499
title: "TEXTSPLIT match_mode 1: probe whether ẞ matches the delimiter ß (and ß the delimiter ẞ), as TEXTBEFORE's does not"
milestone: M7
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTSPLIT with match_mode 1 treats `ẞ` (U+1E9E) and `ß` as Excel does, in both directions, and docs/excel-semantics/text.md
"TEXTSPLIT match_mode 1 case folding" states the rule with probe cells instead of UNVERIFIED.

## Context
- T-4473 (probes/t4473-textsplit-fold-gaps) gave TEXTSPLIT TEXTBEFORE's sigma fold, but left `ẞ` unprobed for TEXTSPLIT.
  TEXTBEFORE's delimiter `ß` does not match a text `ẞ` (probes/textbefore-casefold-length A9), and `fold_for_before_after`
  keeps `ẞ` unfolded for that. TEXTSPLIT's `fold_for_split` (crates/rustsheet-engine/src/functions/text/casefold.rs)
  still folds `ẞ` through SEARCH's `fold_char`. text.md marks the TEXTSPLIT case UNVERIFIED.
- Write one probe case, probes/t4499-textsplit-sharp-s (excel-probe skill, excel-graph, tag M7):
  `TEXTSPLIT("assbẞc","ß",,,1)`, `TEXTSPLIT("aßbẞc","ẞ",,,1)`, `TEXTSPLIT("aẞb","ẞ",,,1)` and the match_mode 0 controls, each
  read through `ARRAYTOTEXT(..,1)` and `COLUMNS(..)`; also TEXTBEFORE's mirror `TEXTBEFORE("aßb","ẞ",1,1)`.

## Acceptance criteria
- [x] probes/t4499-textsplit-sharp-s is committed with oracle values (excel-graph) and every cell passes.
- [x] If TEXTSPLIT matches TEXTBEFORE, `fold_for_split` and `fold_for_before_after` share the `ẞ` arm (no second copy).
- [x] text.md's two case-folding entries drop the `ẞ` UNVERIFIED notes they answer, citing the new cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Every other fold (T-4473), the UTF-16 instance bound (T-4474).

## Outcome
Probe probes/t4499-textsplit-sharp-s (excel-graph): `ß` and `ẞ` match each other in neither direction under match_mode 1, for TEXTSPLIT
(A3, A7, A9) and TEXTBEFORE (A12 is `#N/A`). So TEXTSPLIT now keeps `ẞ` unfolded like TEXTBEFORE: the `ẞ` arm moved into `fold_match_char`,
and `fold_for_split`/`fold_for_before_after` collapsed into one `fold_for_match`. text.md updated. Eval delta: the new probe's 12 cells match Excel,
0 regressions, no known failures added (baseline gains the new case line only).
