---
id: T-3841
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of PIVOTBY"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs --functions PIVOTBY --spill` that raises PIVOTBY's
pairwise argument coverage, and every Excel cell the engine fails is owned by a task.

## Context
- Split from T-3778, which ran the GROUPBY half (pairwise share 0.11 to 1.0; failures owned by T-3837..T-3840). T-3778's
  Context (the dry run, the layout, why opus) applies unchanged; PIVOTBY had pairwise share 0.088 with 4964 unseen pairs on
  main at the start of T-3778.
- Many PIVOTBY cells will fail on the same rules as GROUPBY (scalar inputs, option order, scalar filter, lifted options:
  docs/excel-semantics/arrays.md "GROUPBY scalar inputs, option order and scalar filters in the gen-m7 arg-pairs batch").
  Check whether the GROUPBY owners' rules carry over (one probe row each) and give those cells to T-3837..T-3840 when they do;
  file new tasks otherwise.
- Steps: `make eval`; `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions PIVOTBY --spill`;
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`; `make eval`, `make eval-baseline`. Assign failing cells to
  owners by anchor row (each call's cells sit below its column-A anchor).

## Acceptance criteria
- [x] New gen-m7 cases from `--policy arg-pairs --functions PIVOTBY --spill` are committed with their excel-graph sidecars and 0
      fill failures.
- [x] After `make eval`, PIVOTBY has a `pairwise` share of at least 0.9, or the PR lists the pairs left unseen and why.
- [x] No scored cell is a layout `#SPILL!`.
- [x] Failing cells are known failures in eval/baseline/gen-m7.txt, each owned by an open task; `make exit-owners MILESTONE=M7`
      and `make unowned-off ALL=1` report no new unowned cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells the batch finds (the owning tasks make them).

## Outcome
- **Batch**: 50 new gen-m7 cases from `rustsheet-gen --policy arg-pairs --milestone M7 --functions PIVOTBY --spill` (seed 0, no
  pair skipped), with excel-graph sidecars: 50 written, 0 failures. No oracle cell is a `#SPILL!`, so no scored cell is a
  layout `#SPILL!`.
- **Coverage**: after `make eval`, PIVOTBY's pairwise share is 1.0 with 0 unseen pairs (before: 0.088, 4964 unseen).
- **Eval delta**: gen-m7 grows from 150 to 200 cases and to 93,430 scored cells. 0 regressions; 10,820 new-coverage failures
  recorded by `make eval-baseline` (eval/baseline/gen-m7.txt, visible-parity lock). They sit in 402 calls: 226 anchors are
  visibly off, and the rest are the cells of their spills or lifts. The M7 exit row's spill-extent failures go from 0 to
  10,045; nearly all are T-3890's scalar-filter calls, where Excel returns one `#VALUE!` and the engine spills a table. That is
  an engine rule gap, not the layout, and fixing it is out of scope here, as it was for T-3778.
- **Owners**: the GROUPBY owners T-3837..T-3840 are done, and their fixes cover GROUPBY only, so the PIVOTBY cells get new
  tasks (opus, functions-lookup, `campaign: semantics`). Each task names its calls by a span from the anchor. `make exit-owners
  MILESTONE=M7` reports 0, `scripts/new_case_owners.py` reports every cell owned, and `make unowned-off ALL=1` has no gen-m7 cell
  (its 2,149 are the same as on main).
  - **T-3888** (194 calls, 218 cells): a one-cell error reference as `row_fields` or `col_fields` is `#VALUE!`, and so are
    blank-cell and text `values` (T-3837's rules).
  - **T-3889** (23 calls): an invalid earlier argument's `#VALUE!` wins over a later option's error (T-3838's rule, widened).
  - **T-3890** (14 calls, 10,060 cells): a scalar `filter_array` beside multi-column fields or a header row (T-3839's rule).
  - **T-3892** (169 calls, 507 cells): an early error fills a 2x2 `relative_to` lift.
  - **T-3893** (2 calls, 12 cells): sorting rows by an error-holding values column, and a header joined with an empty first part.
- **Semantics**: docs/excel-semantics/arrays.md gets a new entry, "PIVOTBY in the gen-m7 arg-pairs batch", and the GROUPBY
  entries' T-3841 pointers now name the owners.
- **Also**: T-3887 (sonnet) files the MINOR findings of #2891 (T-3765): probe a blank-cell variable in VSTACK/HSTACK, and name
  T-3765's helpers in arrays.md.
