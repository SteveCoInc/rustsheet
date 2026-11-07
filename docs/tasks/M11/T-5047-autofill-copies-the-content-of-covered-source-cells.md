---
id: T-5047
title: "Autofill copies the content of covered source cells (SPEC-0036 S3)"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-5044]
---

## Goal
Autofill copies a covered source cell's content to the matching covered target cell instead of clearing it.

## Context
Spec: docs/specs/SPEC-0036-merged-covered-content.md, row U6 (rollout S3). `plan_fill_merges` and `fill_covered_cell`
(crates/rustsheet-engine/src/user_model/autofill.rs) assume covered cells hold no content (comments near lines 26-28 and 213): a
target covered cell is cleared and gets only style and link. Change: a non-empty covered source is copied through `Model::fill_cell`
(formulas shifted, with a `SetCellValue` undo diff); it is never a progression term; an empty source still clears. Excel's autofill
over covered content is UNVERIFIED and not probeable on excel-graph (SPEC-0036 X9).

## Acceptance criteria
- [ ] Fill down and fill right over a merged source with covered content copy it (formulas shifted); undo and redo restore it
      (tests in crates/rustsheet-engine/src/test/user_model/test_autofill_merged_cells.rs; update the module doc).
- [ ] A covered source is never used as a progression term (test).
- [ ] docs/excel-semantics/xlsx-storage.md's covered-content entry records the autofill rule as UNVERIFIED (design choice).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Paste (T-5045).
