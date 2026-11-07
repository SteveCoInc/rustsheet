---
id: T-4760
title: "A relative reference in a defined name does not move under row and column inserts and deletes (T-4747 probes)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`Model::displace_defined_names` leaves a relative reference in a defined name where Excel leaves it, so the probes below read what Excel reads.

## Context
T-4747 probed a workbook name `Rel` = `=Data!A5` (relative) used from Calc!A1 and Calc!A3. Excel (excel-graph) does not shift
it under `insert_row 4` or `delete_row 4` on Data (A1 40/60, A3 60/80 = the literal addresses A5, A7 after the edit), while
absolute names do shift (T-4702). The engine shifts it (50/70 on both). Failing cells:
probes/sheet-ops/t4747-name-relative-insert-row.case.json Calc!A1 and Calc!A3; probes/sheet-ops/t4747-name-relative-delete-row.case.json
Calc!A1 and Calc!A3. Code: `displace_defined_names` in crates/rustsheet-engine/src/formula_rewrite.rs. Rule: docs/excel-semantics/references.md.

## Acceptance criteria
- [x] First probe a relative name used from the edited sheet itself (see references.md); the rule so far covers another sheet only.
- [x] The four cells above pass; absolute names (t4702-*) still shift.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe first (acceptance 1): probes/sheet-ops/t4760-name-relative-same-sheet-insert-row and -delete-row (excel-graph,
  2026-11-07; predictions were committed before the fill). A relative name used from the edited sheet does not move
  either: Data!A1 40/60, A3 60/80, and the moved A20 reads 240 both ways. Recorded in references.md "Relative references
  in a name".
- Engine: `Displacement::keep_relative` (stringify/displacement.rs), which `Model::displace_defined_names` sets. Under it,
  a row insert, or a row delete that does not take the referenced row, leaves a wholly relative single-cell reference in
  a name where it is (`Displacement::keeps`). Ranges, column edits, mixed references and a deleted referenced row are
  unprobed and still displace as before (T-4875). Review rounds 1 and 2 (reviewer-excel-fidelity) narrowed the change
  to this. Cell formulas, conditional formats and table edits keep `keep_relative: false`.
- Unit tests: test/user_model/test_relative_name_edits.rs. Under a row edit a relative name stays and an absolute one
  shifts. A column edit, a mixed `$A5`, a relative range and a deleted referenced row still displace. A same-sheet read
  keeps its offset.
- Eval delta: probes known failures 8069 -> 8065. The four T-4747 cells are fixed, the two new t4760 cases pass (8
  cells), and t4702 still passes. 0 regressions.

## Landing notes
- PR #3907, branch agent/engine/T-4760-relative-name-edits. The head is the commit that adds these notes, a round-3 fix on
  top of 586506cc. Not enqueued.
- Review on 586506cc: reviewer-security APPROVE, reviewer-design APPROVE, reviewer-excel-fidelity REQUEST_CHANGES. The
  round-2 block was that the change kept a wholly relative range under a row edit and a single reference on a deleted
  row, and neither is probed.
- The round-3 fix in this commit takes the fidelity reviewer's option (2). `Displacement::keeps(all_relative, row)` now
  keeps only a single-cell, wholly relative reference whose row the row edit does not delete. Ranges go back to the old
  path (`range_corners`). T-4875 and references.md list the forms that are still unprobed. New unit test
  `a_deleted_row_and_a_relative_range_still_displace`.
- Run for this commit: `cargo test -p rustsheet_engine relative_name` (8 passed) and `names` (102 passed), plus
  `cargo fmt`. NOT run: `make check-fast` and the eval check on this head. The probes delta should stay 8069 -> 8065,
  since every probed cell uses a single reference with the edit at row 4.
- Next step: run `make check-fast` with the poi and corpus-bench caches absent, wait for CI, then run round 3 of the
  full panel (design, security, excel-fidelity) on the head. Enqueue once all three approve. This is round 3 of 3: if
  fidelity still blocks, block the task per work-loop step 5.
- Do not widen `keeps` again without probes. Rounds 1 and 2 rejected both widenings.
