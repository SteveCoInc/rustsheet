---
id: T-4446
title: "Defined names and unbound uses with circled or squared letters: Excel's =Ⓐ+1 load refusal and its unusable aⒶ definitions"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1856]
---

## Goal
A defined name or an unbound name use holding a circled or squared letter, or a combining mark, behaves as in Excel: the engine refuses what Excel refuses at load and gives `#NAME?` where Excel does.

## Context
Found by T-1856 (docs/excel-semantics/references.md "Characters of a LET or LAMBDA name", excel-graph single-cell
workbooks, 2026-11-06):
- `=Ⓐ+1` (no such name) is refused at load (501 FileCorruptTryRepair), while `=aⒶ+1`, `=İ+1` and `=नाम+1` load as
  `#NAME?` and `ISREF(aⒶ)` is FALSE; `=`U+1F130`+1` with U+1F130 defined loads as `#NAME?`.
- Defined names `Ⓐ` and `aⒶ` for `Sheet1!$B$1` load when unused, but `=aⒶ+1` with `aⒶ` defined is `#NAME?`. The engine
  resolves such definitions (6).
- A defined name `İ` works (`=İ+1` 6), and the use `i` + U+0307 (decomposed) reads it (6); the engine's lexer stops at
  U+0307, which is not alphanumeric.
- Code: `is_name_text` / `is_valid_identifier` (expressions/utils/mod.rs), defined-name load (defined_names.rs),
  `is_refused_variable_name` (functions/logical/let.rs) has the mark-or-symbol class to reuse.
- The loading forms can be committed as probe cases (the refused one cannot); probe which defined-name characters
  Excel keeps usable before changing the lexer, and whether a squared letter is a first character only for astral
  code points.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the loading forms above; each cell passes, or its rule is recorded in
      references.md with an owner.
- [ ] The refused `=Ⓐ+1` gives the repo's refused-formula error (`#ERROR!`), with a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
