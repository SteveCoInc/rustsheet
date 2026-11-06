---
id: T-4420
title: "PIVOTBY relative_to 3 and 4: share one set for unreached column leaves, scope the empty-set #VALUE! note, probe three fields"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4402]
---

## Goal
PIVOTBY's `relative_to` 3 and 4 sets (`ParentSets`, T-4402) hold one shared set for every column leaf no record
reached, the empty-set `#VALUE!` rule says how far its evidence goes, and three-field intransitive runs are checked
against Excel.

## Context
- Review MINORs on T-4402's PR #3446 (security, design, Excel-fidelity reviewers).
- `ParentSets::prepare` (relative_to.rs) copies the parent records once per column item, so each column leaf
  no record reached (`Grid::unreached`) gets its own full copy. That is items x records per parent, beyond the
  `levels x records` that `check_set_entries` bounds. Have `Grid::column_records` say "every record" and share
  one set for those items (or count them in the check). A `ParentEntry::build` helper can merge the two axis
  branches, and `check_set_entries` can return `()` to callers that do not use the levels.
- `SetId::ParentCell` ids are unique only within one PIVOTBY call; add a note.
- The empty-set `#VALUE!` arm in `Model::aggregate_result` (group_functions.rs) covers every relative set, but its
  evidence is `relative_to` 3 and 4 only (probes/t4382-pivotby-intransitive-paths Calc!C7, D7;
  probes/t4402-pivotby-parent-sets-holdout). Say so in the comment and in coercion.md.
- The hold-out covers two-field axes only. Probe relative_to 3 and 4 with three fields and subtotals at levels 1
  and 2 on intransitive runs (texts as docs/research/t4382/gen_probe.py's), with predictions committed before
  the fill.

## Acceptance criteria
- [x] Column leaves no record reached share one parent set; no eval cell changes.
- [x] The empty-set note is scoped to its evidence.
- [x] A three-field relative_to 3 and 4 probe is filled; its off cells pass or are recorded in coercion.md and owned.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `ParentSets` (relative_to.rs): the column items whose cell holds every record of the line (the Total and the
  leaves no record reached) now share one set per parent. `Grid::column_records` (pivotby.rs) returns `None` for
  them (`ColumnRecords`), and `ParentEntry::build` merges the two axis branches, sharing sets by a `Share` key (the
  parent class for 3, the item for 4, or every record). Before this, the row branch copied the parent's records once
  per such leaf. `check_set_entries` returns `()`; `RelativeSets::new` uses the private `set_levels`. Notes added:
  `SetId::ParentCell` ids are unique within one call; the empty-set `#VALUE!` arm in `Model::aggregate_result` and
  coercion.md are scoped to their `relative_to` 3 and 4 evidence. Unit test `every_record_items_share_one_set`.
- New probe probes/t4420-pivotby-parent-sets-three-fields (excel-graph, 2026-11-06; generator and predictions in
  docs/research/t4420, predictions committed before the fill): 24 runs, three key fields with subtotals at levels
  1 and 2, `relative_to` 3 and 4. 514 of 528 cells match; every `relative_to` 4 cell does. The 14 off cells (B3, B4,
  B7, B8, B10, B11, B12, C3, C4, C7, C8, C10, C11, C12) are a new finding: a parent subtotal no record reached holds
  every record of the line (T-4401's rule, applied to the parent cell). Recorded in coercion.md and owned by T-4428
  (depends on T-4401). A scratch merge of T-4401's branch plus that rule in `Grid::column_records` gives 528 of 528,
  with the t4382, t4402 and t4401 probes all passing.
- Eval: no existing cell changes; the new case is 514/528 (its 14 known failures are in the baseline).
- Not a numeric fit: a probe of a structural rule, its predictions committed before the fill.
