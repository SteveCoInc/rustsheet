---
id: T-4499
title: "TEXTSPLIT match_mode 1: probe whether ẞ matches the delimiter ß (and ß the delimiter ẞ), as TEXTBEFORE's does not"
milestone: M7
lane: functions-text
status: todo
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
- [ ] probes/t4499-textsplit-sharp-s is committed with oracle values (excel-graph) and every cell passes.
- [ ] If TEXTSPLIT matches TEXTBEFORE, `fold_for_split` and `fold_for_before_after` share the `ẞ` arm (no second copy).
- [ ] text.md's two case-folding entries drop the `ẞ` UNVERIFIED notes they answer, citing the new cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Every other fold (T-4473), the UTF-16 instance bound (T-4474).
