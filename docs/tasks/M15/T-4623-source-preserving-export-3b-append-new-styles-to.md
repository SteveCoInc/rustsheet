---
id: T-4623
title: "Source-preserving export 3b: append new styles to a patched package's styles.xml (SPEC-0034 D3.3)"
milestone: M15
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4561]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A cell given a style the source did not have is saved by the patch tier, not by regenerating: the new style
records are appended to the source's `styles.xml`, and every existing record keeps its index.

## Context
Split out of T-4561 (which covers metadata and calcChain).
- Today any change to the style table is a fallback reason (`structure_differs`, "the workbook's styles changed",
  crates/rustsheet-xlsx/src/export/patch.rs).
- SPEC-0034 D3.3 requires the live style table to be a prefix-stable extension of the pristine one: every
  pristine `xf`, font, fill, border, `numFmt` and dxf at the same index.
- That rests on the assumption that the importer keeps the source's `cellXfs` order (`import/styles.rs`). It
  must be checked first.

## Acceptance criteria
- [x] Check that the importer keeps `cellXfs`, fonts, fills, borders and `numFmts` in source order. Cite the code,
      and add a unit test if no test covers it.
- [x] A cell given a new number format patches. Its `xf` and `numFmt` are appended, with their `count`s
      updated, and every other style record is unchanged (unit test).
- [x] A style table that is not a prefix-stable extension of the pristine one falls back to regeneration (unit
      test).
- [x] The round-trip baselines are rerecorded.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cell styles (`cellStyleXfs`, named styles) and dxfs added by conditional-formatting edits.

## Outcome
- **Importer order.** `load_styles` (`crates/rustsheet-xlsx/src/import/styles.rs`) pushes `numFmts`, fonts,
  fills, borders and `cellXfs` in document order, one record per child. A fill it cannot read still pushes a
  default, so indexes hold. The patch does not rely on this alone: `export/patch/styles.rs` re-counts each
  section's records in the source and refuses (regenerates) when the count differs from what the importer read,
  then re-scans what it wrote.
- **Patch.** `export/patch/styles.rs`:
  - `appended` returns the tails of a prefix-stable table. Any changed record, cell style, named style or dxf
    means `None`, which falls back.
  - `patch` appends the new records before each section's close tag, sets `count`, expands an empty `<x/>`, and
    creates `numFmts` as the root's first child when the source has none.
  - The record writers (`font_xml`, `num_fmt_xml`, `cell_xf_xml`) are shared with the regenerating exporter.
- **Tests.**
  - Unit: `patch/tests.rs::styles` covers prefix-stable versus changed tables, the appended counts, a created
    `numFmts`, and a section count mismatch that is refused.
  - Integration: `tests/patched_save.rs::a_new_number_format_and_a_new_font_are_appended_to_the_source_styles`.
- **Baselines.** Rerecorded; every changed entry shrank and none grew. R1 kept rates:
  calipers 87.88% → 98.81%, poi 92.79% → 98.89%, corpus-bench 84.72% → 99.05%, fixtures 89.92% → 93.65%.
  R0 is unchanged.
