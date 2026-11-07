---
id: T-4722
title: "Spec: covered cells of a merged range keep their stored values and formulas, as Excel does"
milestone: M11
lane: engine
status: todo
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
- [ ] A spec in docs/specs/ covers import, evaluation, export and every user-model operation that touches covered
      cells, each rule citing evidence or marked UNVERIFIED with a probe task.
- [ ] Implementation tasks are filed from it (size S/M each), one of them owning the visibly-off cells of
      fixtures/merged-covered-values and fixtures/merged-covered-formula.
- [ ] The implementation tasks own these visibly-off cells (until then this task owns them):
      fixtures/merged-covered-values.xlsx A4:N4
      fixtures/merged-covered-formula.xlsx B1 A3:D3
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the change (the tasks the spec cuts).
