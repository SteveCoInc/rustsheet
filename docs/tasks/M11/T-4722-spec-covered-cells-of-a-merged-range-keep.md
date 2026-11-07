---
id: T-4722
title: "Spec: covered cells of a merged range keep their stored values and formulas, as Excel does"
milestone: M11
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A workbook whose merged ranges store values or formulas in their covered (non-anchor) cells loads, calculates and
saves them as Excel does: formulas read them, covered formulas are recalculated, and a round trip keeps them. This
task writes the spec (docs/specs/) for lifting the engine invariant "covered cells hold no content"; implementation
tasks are cut from it.

## Context
Found by T-4148. Excel keeps covered content (docs/excel-semantics/xlsx-storage.md "A merged range keeps the values
of its covered cells"; fixtures/merged-covered-values, fixtures/merged-covered-formula, excel-graph). Ours: the
importer empties every covered cell (`clear_covered_cells`, crates/rustsheet-xlsx/src/import/merged_cells.rs), and
the engine relies on the invariant in merge (crates/rustsheet-engine/src/merged_cells.rs), autofill
(user_model/autofill.rs) and clipboard (user_model/clipboard.rs). The round-trip census
(docs/research/xlsx-round-trip-2026-11-04.md §2.2) loses 3,604 covered cells of poi/bug66675.xlsx this way (mostly
value-less `t="str"` cells, which are blank in Excel and no longer counted since T-4148) and 12 of poi/60512.xlsm
(G42:K42 and the other row-42 merges store shared strings in covered cells); calipers pv23_056 and corpus-bench 124-46
were not re-measured.
The spec must decide: which engine operations keep, clear or ignore covered content (Excel's UI merge reportedly keeps only the top-left value and a
Format Painter merge keeps them all; neither is probed), what a user edit of a covered cell does,
and how export writes it. Probe the UI-side questions with the excel-probe skill where excel-graph can answer.

## Acceptance criteria
- [x] A spec in docs/specs/ covers import, evaluation, export and every user-model operation that touches covered
      cells, each rule citing evidence or marked UNVERIFIED with a probe task.
- [x] Implementation tasks are filed from it (size S/M each), one of them owning the visibly-off cells of
      fixtures/merged-covered-values and fixtures/merged-covered-formula.
- [x] The implementation tasks own these visibly-off cells (until then this task owns them):
      fixtures/merged-covered-values.xlsx A4:N4
      fixtures/merged-covered-formula.xlsx B1 A3:D3
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the change (the tasks the spec cuts).

## Outcome
docs/specs/SPEC-0036-merged-covered-content.md: the covered-cells-hold-no-content invariant is dropped. Import keeps covered
content (X1, X2 VERIFIED); evaluation, spill blocking and export need no change; a user edit of a covered cell stays refused;
merge, unmerge, range clear, row/column edits, moves and Ctrl+arrow keep their code; paste and autofill stop wiping covered
content. Every row cites evidence or is UNVERIFIED with its probe (T-5048) or the reason it cannot be probed on excel-graph
(paste, autofill, Format Painter and UI merge have no Graph endpoint; M14 is out of scope, decision 0266). The code survey
found that `Model::update_cell_with_*` already writes covered cells (no guard) and that the importer keeps covered links.
Implementation tasks: T-5044 (import, S, sonnet; owns fixtures/merged-covered-values.xlsx A4:N4 and
fixtures/merged-covered-formula.xlsx B1 A3:D3), T-5045 (paste, M), T-5047 (autofill, S), T-5048 (excel-graph probes, M).
No probe was run here: the UI-side questions are not answerable on excel-graph, and the structural ones need a merged-range
case builder, which T-5048 owns. Eval delta: none (docs and tasks only).
