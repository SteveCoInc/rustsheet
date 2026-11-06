---
id: T-4559
title: "Source-preserving export 1: retain the source and write it back verbatim when nothing changed (SPEC-0034 D1, D2)"
milestone: M15
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4183]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
`rustsheet::Workbook` keeps the bytes it was loaded from. Saving a workbook whose cells and structure are unchanged
since load writes every part of the source back byte for byte (SPEC-0034 D1, D2 "Unchanged"). Any other workbook
keeps today's regenerating export.

## Context
SPEC-0034. The save-time diff re-imports the retained source into a pristine `Model` and compares it with the live
one, using the typed model's own fields (D2.1). This first slice only needs the Unchanged decision. Classifying
Content against Structural, and acting on it, is T-4560.

R0 recalculates. A workbook whose recalculation reproduces every cached value is Unchanged, so in
`roundtrip-fixtures` such cases become fully kept. That is the first big step towards the owner's 99% goal.

## Acceptance criteria
- [x] Load then save, with no edit, of a workbook whose recalculation changes no cached value gives the source's
      parts, every part byte-identical (unit test).
- [x] One edited cell, or one changed cached value, falls back to regeneration (unit test).
- [x] The round-trip baselines are rerecorded. Their kept-item rate rises, reported in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Patching (T-4560).

## Outcome
`rustsheet::Workbook` keeps its source package and the import options (`Source`, one `import` path shared by the
load and the save's pristine copy). `to_xlsx_bytes` calls `rustsheet_xlsx::export::save_preserving`:
- when the live model's `Workbook` equals the pristine re-import, the source is written back byte for byte
  (`SaveTier::Verbatim`);
- otherwise it regenerates as before (`Regenerated`).

The tier is returned for T-4562. Tests (`crates/rustsheet/tests/verbatim_save.rs`):
- an unchanged workbook, with or without recalculation, is its source byte for byte;
- an edit regenerates;
- a stale cached value regenerates when recalculated, and is kept when not.

Measured with this change:
- `roundtrip-fixtures` 3,377 → 3,375 known failures: `rich-error-export` is now verbatim. The other fixtures carry
  placeholder caches that recalculation changes, which T-4560's patcher handles.
- Ad hoc runs (uncommitted suites, T-4173 commits them), with the parts, elements and attributes of SPEC-0033:
  - the 83 Calipers `roundtrip/` Excel goldens keep 99.68% of 554,447 items;
  - POI keeps 99.84% of 3.83M items, with 292 of 336 loadable workbooks verbatim.

No cell eval delta.

