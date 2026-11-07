# SPEC-0036 — Covered cells of a merged range keep their content

- **Status:** accepted (2026-11-07, T-4722)
- **Milestone / lane:** M11 / engine
- **Tasks:** T-4722 (this spec); rollout T-5044 (S1), T-5045 (S2), T-5047 (S3), T-5048 (S4)

## Problem
Excel keeps a value or formula stored in a covered (non-anchor) cell of a `<mergeCell>` range, reads it in formulas
and recalculates it (docs/excel-semantics/xlsx-storage.md "A merged range keeps the values of its covered cells";
fixtures/merged-covered-values, fixtures/merged-covered-formula, excel-graph 2026-11-06, T-4148). We empty every
covered cell on import (`clear_covered_cells`, crates/rustsheet-xlsx/src/import/merged_cells.rs:156, called from
`load_sheet` in import/worksheets.rs), because the engine treats "covered cells hold no content" as an invariant.
The cost, measured:
- 15 visibly-off cells in the two fixtures (eval/baseline/fixtures.txt: merged-covered-values A4:N4,
  merged-covered-formula B1 A3:D3).
- The round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2) loses 12 covered cells of poi/60512.xlsm
  (row 42's merges store shared strings in covered cells) and the covered cells of poi/bug66675.xlsx (3,604 cells,
  mostly value-less `t="str"` cells, blank in Excel and no longer counted since T-4148).

## Goals / non-goals
Goals: import, evaluation and export keep covered content as Excel does; every user-model operation that reads or
writes covered cells has a stated rule, either evidenced or marked UNVERIFIED with the probe task that settles it.
Non-goals: changing the UI merge (`merge_cells`), letting a user edit a covered cell, or merge-aware navigation.

## Excel behavior to match
| # | Rule | Evidence |
|---|---|---|
| X1 | A covered cell's stored value is read by references and ranges as stored: `=B1` "x", `=COUNTA(A1:D1)` 4, `=SUM(A1:D1)` 7, `=COUNTBLANK(A1:D1)` 1; a value-less `<c t="str"/>` is blank | fixtures/merged-covered-values (VERIFIED, excel-graph) |
| X2 | A covered formula is recalculated and is a formula: `=A1+1` in B1 of A1:B1 gives 2, `ISFORMULA(B1)` TRUE, `=COUNT(A1:B1)` 2 | fixtures/merged-covered-formula (VERIFIED, excel-graph) |
| X3 | A spill into any cell of a merged range, covered or anchor, empty or not, is `#SPILL!` | implemented (`model.rs:993`, test `spill_into_merge_is_spill_error`), but no probe in docs/excel-semantics evidences it: UNVERIFIED, unchanged here, probed in T-5048 |
| X4 | Saving keeps covered content | UNVERIFIED: no case checks what Excel writes for a covered cell on save. Owned by T-5048 |
| X5 | The UI cannot select or edit a covered cell (selection snaps to the anchor) | UNVERIFIED, and not probeable on excel-graph (no UI). We keep refusing the edit (U1) |
| X6 | Deleting the anchor's row or column of a merge with covered content: the merge shrinks and the first surviving covered cell is its anchor, showing its stored value | UNVERIFIED; excel-graph replays `delete_row`/`delete_column` (docs/architecture/oracle.md "Sheet ops"), so T-5048 probes it |
| X7 | Merge (Home > Merge) keeps only the upper-left value; Format Painter's merge keeps every covered value | UNVERIFIED and not probeable (no Graph endpoint for Format Painter; Range `merge` is unprobed); we keep today's merge (U2) |
| X8 | Unmerge shows the covered content again | UNVERIFIED; Graph has Range `unmerge`, but the oracle's `sheet_ops` do not replay it. T-5048 decides whether adding it is in reach |
| X9 | Paste, autofill and clear over covered content | UNVERIFIED; no Graph endpoint replays paste or autofill (desktop questions; M14 is out of scope, decision 0266). The rules below follow the storage model: what is stored stays unless the operation writes the cell |

The "follows the storage model" default for X6, X8 and X9 is a design choice, not evidence: each rule says so, and a
probe that contradicts it changes the rule.

## Design
The invariant is dropped. A covered cell is an ordinary stored cell that the UI does not select. Each operation:

| Op | Today | After | Rule basis |
|---|---|---|---|
| **I1 import** | `clear_covered_cells` empties covered cells (keeps style) | keep them as stored; delete `clear_covered_cells` and its index-based helper if nothing else uses it (`KeptRanges` stays: `load_merge_cells` uses it) | X1, X2 |
| **E1 evaluation** | no merge logic; reads the stored cell | unchanged (already right once I1 keeps the content) | X1, X2 |
| **E2 spill blocking** (`model.rs:993`) | any merged cell blocks | unchanged | X3 |
| **W1 export** (`export/worksheets.rs`) | writes cells as stored | unchanged; S1 adds a round-trip test | X4 |
| **U1 `set_user_input` on a covered cell** (`prepare_cell_for_user_input`, `model.rs:1855`) | refused | unchanged (refused); the raw `update_cell_with_*` API already writes covered cells and stays | X5 |
| **U2 merge** (`merge_cells`) | refuses over two content cells; moves the one content cell to the anchor; clears the rest | unchanged | X7 (UNVERIFIED) |
| **U3 unmerge** | drops the range; cells untouched | unchanged: covered content shows again | X8 (UNVERIFIED) |
| **U4 range clear** (`range_clear_contents`/`range_clear_all`) | clears every cell of the range | unchanged; a selection of a merged cell is the whole range, so its covered content is cleared too | X9 (UNVERIFIED) |
| **U5 paste** (`clipboard.rs`, cut 390-460, copy 630-670) | recreates source merges with `merge_cells_keep_styles`, which clears covered positions, including content the paste just wrote | covered cells inside the pasted block keep what the paste wrote; only cells a recreated merge swallows outside the pasted block are cleared (S2) | X9 (UNVERIFIED) |
| **U6 autofill** (`autofill.rs`, `fill_covered_cell` 237-279) | target covered cells are cleared and get style and link only; covered sources are skipped as progression terms | a non-empty covered source is copied to its target (formulas shifted as `fill_cell` does), never used as a progression term; an empty source still clears (S3) | X9 (UNVERIFIED) |
| **U7 insert/delete rows and columns** (`displace_merged_cells`, actions.rs) | the first surviving covered cell becomes the anchor (empty today) | unchanged code; with content it shows the stored value. S1 adds a test; S4 probes it | X6 (UNVERIFIED) |
| **U8 move rows/columns** | content moves with its cells | unchanged | — |
| **U9 Ctrl+arrow** (`navigate_to_edge_in_direction`) | judges emptiness at the anchor | unchanged | UNVERIFIED, UI-only |
| **U10 undo/redo** | merge diffs record covered clears as `SetCellValue ""` | unchanged; S2 and S3 record what they write | — |

Hyperlinks: the importer already keeps a covered cell's link (it loads links after the clear), so I1 makes value
and link consistent; `merge_cells` still clears both.

Resource limits (directive 5): I1 removes work (one pass over covered positions per merge); nothing new is
allocated. The importer's cap on covered positions it maps to anchors (xlsx-storage.md "Import resource limits")
is unchanged. The clipboard's clamp of a copy to the sheet dimension (`clipboard.rs:145`) stays sound: covered
content is stored, so it is inside the dimension.

## Alternatives considered
- **Keep the invariant, load covered content into a side table** for export only. Rejected: X1 and X2 need formulas
  to read and recalculate it, which a side table would have to special-case in every reference path.
- **Redirect a covered-cell edit to the anchor.** Rejected: nothing evidences it, and the UI never selects a covered
  cell; the refusal is the conservative behaviour.

## Test plan
- Eval: fixtures/merged-covered-values A4:N4 and fixtures/merged-covered-formula B1 A3:D3 pass (S1); `make
  eval-baseline` removes their known-failure lines. The roundtrip suite reports no covered-cell loss for
  poi/60512.xlsm row 42.
- xlsx tests: `test_import_sanitizes_merged_cells` asserts covered C2 keeps 43;
  `many_whole_column_merges_import_quickly_and_clear_covered_cells` becomes `..._keep_covered_cells`, same time
  bound, asserting the stored values; a round-trip test reloads covered values and formulas unchanged.
- Engine tests (with undo/redo each): unmerge shows covered content; a range clear of a merged range clears it;
  deleting the anchor row surfaces the covered value; copy- and cut-paste keep pasted covered content (S2);
  autofill copies a covered source (S3).
- Tests that assert today's emptiness and stay true because merge still clears (`merge_moves_*`,
  `reference_to_covered_cell_is_zero`) are unchanged.

## Rollout
1. **S1 (T-5044, sonnet, S):** I1 plus the W1, U3, U4 and U7 tests; owns the 15 fixture cells.
2. **S2 (T-5045, opus, M):** U5, after S1.
3. **S3 (T-5047, opus, S):** U6, after S1.
4. **S4 (T-5048, opus, M):** probe X3, X4, X6 and, if the oracle can replay `unmerge`, X8 on excel-graph; record the
   findings and amend U3/U7 if Excel differs.
Until S2 and S3 land, a paste or autofill over imported covered content drops it, as all covered content is dropped
on import today, so S1 alone is no regression.
