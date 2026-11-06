---
id: T-4513
title: "Round-trip semantic model: cell, sheet, name and feature items with resolved styles and indexes (SPEC-0033 D2, D3)"
milestone: M15
lane: harness
status: done
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
- [x] Identity holds on the fixtures suite.
- [x] Invariance: shuffled shared strings and renumbered style records compare `kept`.
- [x] Sensitivity: one changed value, one changed formula and one changed style property are each found.
- [x] V7 fails a crafted out-of-range shared-string index. An out-of-range style index is refused by the importer, so V8 fails it.
- [x] The fixtures baseline is updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Features the model does not hold, which are scored as package items (T-4509).

## Outcome
`roundtrip/semantic.rs` scores SPEC-0033's semantic items as keyed facts (`cell Sheet!A1 style`,
`merge Sheet!A1:B2`, `name Sheet!X`, `value Sheet!A1`, ...).

Sources of the facts:
- styles (resolved `Style`, never an index) and structure come from the importer's `Model`: sheet order, state, tab
  colour, frozen panes, grid lines, merges, hyperlinks, CF rules, defined names, tables;
- formula text and constants come from the snapshot reader, the file's own text, so a printer bug cannot hide behind
  the engine printing both sides alike (X5's leading whitespace is dropped);
- cached values of formula and spill cells compare the saved, recalculated workbook with the output as loaded, and
  errors compare by kind.

V7 flags an out-of-range shared-string index. An out-of-range style index is refused by the importer, so V8 already
fails it.

`roundtrip-fixtures`: 18,796 items, 3,377 known failures. The new ones are 210 `formula changed` items, the
external-reference printing bug (T-4140).

Formula text compares as stored, canonicalized as Excel stores it: X5's leading spaces, TABs and LFs, and an
`_xlfn.` prefix the function does not need (T-0172). SPEC-0033 D3.1 was amended to say so (fidelity review).

Known gaps of the `_xlfn.` canonicalization:
- Excel's re-store without the prefix is evidenced for CEILING only (T-0172); for the other classic functions Excel
  is only known to accept it on load. T-4503's probe of what Excel rewrites on save covers it.
- Because the canonicalization uses the export's own table, an export that wrongly drops a prefix a newer function
  needs canonicalizes both sides to the bare name and is not seen here. The export-list check against XlsxWriter
  (T-0412) guards that.

Not covered yet:
- the tint tolerance of D3.4: tints compare exactly, which is harmless while outputs are ours, but needed once
  Excel-written outputs are compared (T-4180);
- cell protection, which `Style` does not hold, so only the package diff sees it, as `cellXfs/xf/protection`
  element items. Modelling it is M15's "protection" feature (milestones.md scope), and SPEC-0034's patch tier keeps
  it byte for byte meanwhile.

Carried from the reviews of #3529 and #3551:
- `xmldiff::compare` borrows the local path and builds no item path past `MAX_ITEMS`;
- per-row `CT_` comments;
- the xlsx-storage.md `Status:` line stands apart from its paragraph;
- `SheetRef` has named fields.

Tests:
- shuffled shared strings and renumbered style records compare `kept`;
- a needless `_xlfn.` prefix is no difference, and a needed one is;
- a changed constant, formula and style property are each found;
- V7 on an out-of-range shared string, and the importer's refusal of an out-of-range style.

No cell eval delta.

