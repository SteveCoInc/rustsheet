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
- **`export/ownership.rs`** (new):
  - D5.1's table, one list of relationship types per policy, with digital-signature certificates under `Drop`;
  - `policy_of(rel_type, content_type, reached_from)` with its unknown defaults. A part no relationship reaches is known
    by its content type alone;
  - `Graph`, the parts and internal relationships of a package read within the load's limits, with the workbook's
    `sheetId`s and `cacheId`s;
  - `closure` within D5.6's depth (16) and size (1,000) limits.
- **`export/ownership/decide.rs`** (new):
  - **Roots** are the parts reached from the package, the workbook or a sheet (chartsheets, dialogsheets and
    macrosheets included), and orphans. Any other part takes the verdict of the units whose closures hold it.
  - **Coupling groups** are per sheet:
    - the legacy drawing group: VML, comments, threaded comments, ctrlProps, ActiveX, OLE, and a drawing holding
      `a14:compatExt`;
    - the slicer and timeline group, joined to the caches its parts name (`cache` against the cache definition's
      `name`).
  - Groups that share a part are merged.
  - **The shared-part rule:** a part a keepable unit holds is keepable; otherwise a limit wins over a closure reaching
    regenerated content.
  - **Bounds:** one cached probe per part, at most 1 MiB of it. One closure per root. A total work cap of 10^6 closure
    parts, which carries to every remaining unit. An unreadable part makes its unit `LimitExceeded`.
- **Reasons:**
  - a modelled part the output lacks: `NotPreserved`;
  - the `Drop` policy: `Policy`;
  - a closure reaching what the export regenerates (a modelled or dropped part, or a chart holding `c:pivotSource`):
    `ClosureReachesRegenerated`;
  - a limit: `LimitExceeded`;
  - VBA: `NotPreserved` until T-5010, since nothing writes macro-enabled packages yet;
  - a keepable part: `NotPreserved` until its writer lands (T-5000 onward).
- **Written or not.** The output is checked for a part with the same identity. A part's identity is its owner's identity
  plus the relationship kind, with a label the export keeps:
  - a worksheet's `sheetId`, which renames, moves and dropped macro or chart sheets keep. Positions were tried first and
    misnamed written parts: poi xlmmacro.xlsm, and excel-saved t4187's `.rels` order;
  - a table's or pivot table's `name`;
  - a pivot cache's set of pivot-table names, since the export renumbers `cacheId`;
  - otherwise an ordinal among siblings of that kind. When a save writes fewer siblings, the last ones are reported,
    which can name the wrong one. SPEC-0033 D7.3 scores that `changed`, so the baseline ratchet catches it.
- **Deviations, recorded in SPEC-0034:**
  - the legacy group takes every `vmlDrawing` of a sheet, the header and footer one included;
  - each package read has its own budget;
  - the regenerated tier is buffered.
- **Plumbing.** `preserving.rs` calls the decisions directly. `report.rs` keeps the patched tier and `unreadable`. The zip
  and `.rels` helpers live in `import/package.rs`, and `rel_kind` is shared with `pivot_drawings.rs`. `XmlNode`'s
  `relationship_id` and the patcher's `attr_span` are reused.
- **Tests** (`ownership/tests.rs`):
  - every policy and the unknown defaults;
  - closures with an external target, and the depth and size limits;
  - a modelled part the output lacks, including the middle of three tables, named by name;
  - a part reached twice;
  - a non-root chart that sorts before its drawing, and an image shared with a clean root;
  - the legacy group with a pivot chart;
  - per-sheet slicer and timeline groups joined by cache, with overlapping groups merged;
  - the work limit, and an unreadable drawing;
  - a sheet matched by `sheetId` after a move.
- **Baselines** are rerecorded with no regression, and no item scores "the save report names it dropped, but it was
  written".
- **Review round 1:**
  - security: quadratic work, uncached probes, failed reads treated as "no";
  - design and Excel fidelity: roots, the shared-part rule, per-sheet and merged groups, identities, duplicated helpers.

  All are addressed above.
