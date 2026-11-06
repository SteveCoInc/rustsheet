---
id: T-4623
title: "Source-preserving export 3b: append new styles to a patched package's styles.xml (SPEC-0034 D3.3)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-4561]
---
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
- [ ] Check that the importer keeps `cellXfs`, fonts, fills, borders and `numFmts` in source order. Cite the code,
      and add a unit test if no test covers it.
- [ ] A cell given a new number format patches. Its `xf` and `numFmt` are appended, with their `count`s
      updated, and every other style record is unchanged (unit test).
- [ ] A style table that is not a prefix-stable extension of the pristine one falls back to regeneration (unit
      test).
- [ ] The round-trip baselines are rerecorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cell styles (`cellStyleXfs`, named styles) and dxfs added by conditional-formatting edits.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
