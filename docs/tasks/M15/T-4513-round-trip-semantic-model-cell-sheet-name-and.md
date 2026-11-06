---
id: T-4513
title: "Round-trip semantic model: cell, sheet, name and feature items with resolved styles and indexes (SPEC-0033 D2, D3)"
milestone: M15
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-4172]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
The round-trip suite scores the semantic items of SPEC-0033 D2 and D3: cells (value and type, formula text, resolved
style properties), sheets, defined names and the features the model holds (merges, hyperlinks, conditional
formats, tables, frozen panes). Indexes are resolved, never compared, and validator check V7 runs.

## Context
SPEC-0033 D2, D3 and D6 (V7). Build the canonical feature model from the importer's typed model, not from a second
xlsx parser (CLAUDE.md conventions: tools never re-implement xlsx value parsing). The snapshot reader is the
harness's existing reader of stored values; extend it rather than add another. A style compares as its resolved
properties, with tints to 1e-6 relative (T-0419). An empty-string cell is not a blank cell (T-4148).

Which pair a field compares:
- cell inputs (constant value, or formula text) and resolved styles compare the source as loaded against the output
  as loaded;
- cached values compare the recalculated model that was saved against the output as loaded.

So an engine-parity difference is scored by the cell suites, not reported here as an I/O loss.

Carried from the reviews of #3551 and #3529:
- borrow the local path in `xmldiff::compare` instead of cloning it, and build no item path once `MAX_ITEMS` is
  reached;
- add the missing per-row `CT_` comments (`definedName`, `row`, `col`, `sheetView`, `workbookView`,
  `sheetFormatPr`);
- separate the `Status:` line from the paragraph under xlsx-storage.md "Round trips";
- give the snapshot reader's `SheetRef` named fields.

## Acceptance criteria
- [ ] Identity holds on the fixtures suite.
- [ ] Invariance: shuffled shared strings and renumbered style records compare `kept`.
- [ ] Sensitivity: one changed value, one changed formula and one changed style property are each found.
- [ ] V7 fails a crafted out-of-range style index.
- [ ] The fixtures baseline is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Features the model does not hold, which are scored as package items (T-4509).
