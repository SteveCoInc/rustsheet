---
id: T-4757
title: "Sheet names with a later mark: INDIRECT of unquoted á!B1, a cell-shaped B1+U+0301 sheet, and an Mc mark (probe the T-4446 inferences)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine's sheet-name quoting and INDIRECT's unquoted-prefix check agree with Excel for every mark class after the
first character, on evidence rather than the inference T-4446 made from U+0301 and U+20DD.

## Context
From the T-4446 Excel-fidelity review (PR #3810, MINOR). T-4446 probed (probes/t4446-defined-name-characters A19:A23)
that FORMULATEXT writes a reference to sheet `a`+U+0301 (Mn) or `x`+U+20DD (Me) unquoted, and `name_needs_quoting`
(expressions/utils/mod.rs) now takes every `\p{M}` after the first character. Unprobed:
- INDIRECT of the unquoted text `á!B1` (decomposed): `is_indirect_unquoted_sheet` still takes only `is_alphanumeric`;
- a spacing mark (Mc, e.g. `क`+U+093E as a sheet name) and a non-alphabetic Mc;
- a cell-shaped sheet name with a mark, `B1`+U+0301 (`parse_reference_a1` decides quoting there); compare `C1`+U+0301,
  which Excel refuses as a defined name (references.md "Characters of a name").

## Acceptance criteria
- [x] A probe case (excel-graph) covers the three forms with FORMULATEXT and INDIRECT; each cell passes, or its rule is
      recorded in references.md with an owner.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- New probe probes/t4757-later-mark-sheet-names (excel-graph, 2026-11-07, 20 cells). Its predictions were committed
  before the fill, and every one held. It covers five sheets:
  - `a` + U+0301 (Mn);
  - `क` + U+093E (Mc, alphabetic);
  - `x` + U+0F3E (Mc, not alphabetic);
  - the cell-shaped `B1` + U+0301;
  - the R1C1-shaped `C1` + U+0301.
  Each is read through a quoted reference, through its FORMULATEXT, and through INDIRECT of the unquoted and of the
  quoted text.
- Excel and ours agree on 19 of 20 cells:
  - FORMULATEXT writes the first four unquoted and `C1` + U+0301 quoted;
  - INDIRECT of the unquoted text reads the sheet for the first four, including the decomposed `á!B1` and the
    non-alphabetic Mc;
  - quoted INDIRECT reads all five.
- The one difference is `INDIRECT("C1́!B1")` (Sheet1!A19): Excel gives `#REF!`, ours reads 23. Owned by T-4861
  (engine, sonnet). It probes which other R1C1-shaped unquoted prefixes Excel refuses before changing the engine.
- docs/excel-semantics/references.md "Characters of a name", sheet-names bullet: the T-4446 inference is now
  evidenced, and the C1 + U+0301 rule is recorded.
- Eval delta: probes +1 case (20 cells), 19 passing, 1 known failure owned by T-4861; 0 regressions
  (`make eval-baseline`).
