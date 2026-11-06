---
id: T-4420
title: "PIVOTBY relative_to 3 and 4: share one set for unreached column leaves, scope the empty-set #VALUE! note, probe three fields"
milestone: M7
lane: engine
status: todo
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
- [ ] Column leaves no record reached share one parent set; no eval cell changes.
- [ ] The empty-set note is scoped to its evidence.
- [ ] A three-field relative_to 3 and 4 probe is filled; its off cells pass or are recorded in coercion.md and owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
