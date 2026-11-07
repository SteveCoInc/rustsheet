---
id: T-4999
title: "Stage 5: ownership matrix and closure rule for regenerated saves (SPEC-0034 D5.1, D5.2, D5.6)"
milestone: M15
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4562]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
One ownership table, one closure computation and one coupling-group detector decide, for every part of the source
package, whether a regenerated save keeps it. Every part that is not kept is reported.

## Context
SPEC-0034 D5.1 (the matrix), D5.2 (coupling groups, the closure rule, the `c:pivotSource` exclusion) and D5.6 (limits,
with the deterministic drop order). The table lives in `export/ownership.rs`, keyed by relationship type and then by
content type, never by part name. This task computes the decision and reports it. Writing the parts is T-5000 onward.
The package plumbing already exists: `PartNames`, `parse_relationships` and `resolve_target`
(crates/rustsheet-xlsx/src/import/package.rs).

## Acceptance criteria
- [x] `export/ownership.rs` holds the D5.1 table, one row per relationship type listed there.
- [x] `policy_of(rel_type, content_type, reached_from)` gives the unknown defaults (package/workbook-reached, or
      sheet-reached).
- [x] `closure(source, root)` follows internal relationships only, within the D5.6 depth and size limits.
- [x] The legacy-drawing and slicer coupling groups are detected per sheet, as D5.2 defines them.
- [x] Each source part of a Regenerated save with a source gets a decision: kept, or dropped with a SPEC-0033 D7 reason.
      The drops appear in the SaveReport (T-4562).
- [x] Limits are applied in source part-name order, so the result is deterministic.
- [x] A `Modelled` part the regenerating export did not write is reported `NotPreserved`, found by comparing the output's
      own inventory, not assumed written. T-4562 leaves these unreported: a pivot cache `PivotPlan` drops (calipers
      pv23_045), rich values that are not rich errors, and a table (poi SingleCellTable).
- [x] The duplicated zip and `.rels` helpers of `export/report.rs` (`entry_names`, `rels_owner_folder`, `is_rels`) move
      next to `PartNames` in `import/package.rs` and are shared with the matrix.
- [x] Unit tests cover:
  - each policy, and the unknown defaults both ways;
  - a closure that reaches a `Modelled` part, and a chart holding `c:pivotSource`;
  - a shared part, and an external target;
  - each coupling group;
  - each limit.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing kept parts (T-5000, T-5002, T-5008 to T-5010).
- Fragments (T-5001, T-5003).

## Outcome
- **`export/ownership.rs`** (new) holds:
  - D5.1's table, `policy_of(rel_type, content_type, reached_from)` with its unknown defaults, and `Graph`, the
    package's parts and internal relationships read within the load's limits;
  - `closure` (D5.2) within D5.6's depth (16) and size (1,000) limits;
  - the coupling groups: per sheet, the legacy drawing group (VML, comments, threaded comments, controls, OLE, and a
    drawing holding `a14:compatExt`), and the workbook's slicer and timeline group;
  - `regenerated`, which decides every source part. Units are decided in part-name order: a group's closure as one
    unit, and a chart holding `c:pivotSource` counts as regenerated.
- **Reasons:**
  - a modelled part the output lacks: `NotPreserved`;
  - the `Drop` policy: `Policy`;
  - a closure reaching what the export regenerates: `ClosureReachesRegenerated`;
  - a limit: `LimitExceeded`;
  - VBA: `NotPreserved` until T-5010, since nothing writes macro-enabled packages yet;
  - a part the matrix would keep: `NotPreserved` until its writer lands (T-5000 onward).
- **Written, or not.** A part counts as written when the output has a part with the same identity: its owner's
  identity, the relationship kind and its ordinal among the owner's relationships of that kind. A worksheet is
  identified by its `sheetId`, which renames, moves and dropped macro or chart sheets all keep. Positions were tried
  first and misnamed written parts: poi xlmmacro.xlsm's macrosheet sits before Sheet2, and a source's `.rels` can list
  sheets in any order (excel-saved t4187).
- **`report.rs`** uses the matrix for the regenerated tier and for the patched tier's calcChain. The zip and `.rels`
  helpers (`zip_entry_names`, `rels_owner_folder`, `rels_owner_part`, `is_rels`) moved to `import/package.rs`. The
  regenerated tier is now written to memory first, so its own inventory can be read.
- **Tests:**
  - every policy and the unknown defaults;
  - a closure through a chart reaching a table, an external target, and the depth and size limits;
  - a modelled part the output lacks;
  - a part reached twice;
  - the legacy group with a pivot chart, and the slicer group;
  - a sheet matched by `sheetId` after a move.
- **Baselines.** Rerecorded with no regression; one poi entry shrank. No item anywhere scores "the save report names
  it dropped, but it was written". Scores change only where an accepted reason applies, which stays `Policy` for now.
