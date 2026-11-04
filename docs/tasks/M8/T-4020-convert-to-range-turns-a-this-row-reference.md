---
id: T-4020
title: "Convert to range turns a #This Row reference off the data rows into #REF!, not a detached reference"
milestone: M8
lane: engine
status: todo
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
After `convert_table`, a `#This Row` reference off the table's data rows reads `#REF!` with FORMULATEXT `=#REF!`, and
the xlsx export writes `<f>#REF!</f>`, as Excel does; a table re-created under the same name does not take it back.

## Context
- Evidence (T-3629, excel-graph 2026-11-04; references.md convert-to-range entry): after Graph `convertToRange`,
  F1 and F9 (`=Sales[[#This Row],[Qty]]` above and below the table) are `#REF!` with FORMULATEXT `=#REF!`, and
  Excel's saved file holds `<f>#REF!</f>` for them; a data-row one (H5) is lowered to `$C5`. Re-adding a table
  named Sales over the same area or another (`add_table`) leaves them `#REF!`.
- Owned cells (all four in each): probes/table-edits/t3629-convert-plain F1 F2 F9 F10;
  probes/table-edits/t3629-convert-names F1 F2 F9 F10; probes/table-edits/t3629-save-detached F1 F2 F9 F10;
  probes/table-edits/t3629-readd-same-area F1 F2 F9 F10; probes/table-edits/t3629-readd-other-area F1 F2 F9 F10.
- The engine keeps them as detached references (`StructuredReference::detached`, T-2763: `#VALUE!`, `=Sales[@Qty]`,
  export `#VALUE!`). T-2763's evidence, probes/table-edits/table-convert-names-this-row (2026-10-25), holds a
  `#This Row` of Sales on sheet T: with such a cross-sheet reference present, Graph's `convertToRange` left the
  table in place (T-3629 saw the saved file still define Sales on B3:C7), so that case's F1, F9 and T!A1 most
  likely read a live table, not a converted one. Re-running it on 2026-11-04 crashed Excel's recalculation
  (`InvalidSession`, `InternalErrorCrash`) twice; its sidecar was kept.
- Likely change: lower an off-row `#This Row` to `#REF!` at convert (as `rewrite_table_references` does for a
  deleted table) and retire `detached` if nothing else needs it; decide what table-convert-names-this-row's and
  table-convert-cross-sheet-this-row's sidecars mean once T-4021 can tell a no-op convert.

## Acceptance criteria
- [ ] The five t3629 cases above pass on F1, F2, F9 and F10; the xlsx export writes `#REF!` for such a cell.
- [ ] table-convert-names-this-row and table-convert-cross-sheet-this-row are re-probed with a convert that took
  effect (T-4021) or recorded as no-op converts; references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel (M14).
