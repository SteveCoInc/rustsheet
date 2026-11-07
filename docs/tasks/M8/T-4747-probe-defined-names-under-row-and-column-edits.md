---
id: T-4747
title: "Probe defined names under row and column edits: relative references, sheet-scoped names, moves, and the stored spelling of a rewritten name (T-4702 follow-up)"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the corners of T-4702's rule ("Row and column inserts and deletes rewrite defined names",
docs/excel-semantics/references.md) that are still unprobed, and an engine change wherever it differs.

## Context
- T-4702 probed absolute, workbook-scoped names on the edited sheet (probes/sheet-ops/t4702-*) and made
  `Model::displace_defined_names` (crates/rustsheet-engine/src/formula_rewrite.rs) rewrite every name for row and column
  inserts and deletes. Its excel-fidelity review listed what is still unprobed:
  - a name with relative references (`=Data!A1`), which the engine anchors at A1 of the edited sheet, while
    `rewrite_formulas` (sheet rename and delete) anchors names at A1 of the first remaining sheet;
  - a sheet-scoped name, and an unqualified reference in a name;
  - row and column moves (`DisplaceData::RowMove`/`ColumnMove`), which leave names alone in the engine;
  - the stored spelling of a rewritten name: the engine stores `#REF!` and `S!$C$1:$C$1`, which reaches xlsx export.
- Avoid names that are cell addresses (`NA1` is column NA, row 1; references.md "Probe trap").

## Acceptance criteria
- [ ] Probe cases (excel-probe skill, predictions committed before the fill) cover each item above. For the spelling,
      use a round trip of the saved workbook's definedName text.
- [ ] references.md records the results. Each mismatch is fixed here if it is S-sized, or gets an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats (T-4702 covers their undo).
