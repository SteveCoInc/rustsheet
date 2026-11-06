---
id: T-4560
title: "Source-preserving export 2: patch changed cells into the source worksheets, appending shared strings (SPEC-0034 D3.1-D3.4)"
milestone: M15
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4183, T-4559]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A workbook whose structure is unchanged but some of whose cells changed (edits, or recalculated cached values) is
saved by patching the source. Only the changed `<c>` elements are rewritten, new shared strings are appended, and
every other byte and part is kept (SPEC-0034 D3.1, D3.2, D3.4, plus the D3.3 shared-string part).

## Context
SPEC-0034 D3:
- the streaming worksheet pass;
- cells written by the export's own cell writer;
- shared-formula members whose master changed are broken out of the group;
- out-of-order or reference-less sheets fall back to regeneration.

Bounded per directive 5.

## Acceptance criteria
- [x] R1-style unit tests: a changed constant, a changed formula, a new cell in an existing row, a new row, and a
      cleared cell each patch only those cells, and every other part is byte-identical.
- [x] Each fallback trigger regenerates (unit test).
- [x] A new string is appended to `sharedStrings.xml`, and `count` and `uniqueCount` are updated.
- [x] The round-trip baselines are rerecorded.
- [x] `make check` green; reviewers approve.

## Out of scope
- New styles and `metadata.xml` (T-4561).

## Outcome
- `export/patch.rs` adds the patch tier. When the structure is unchanged, `save_preserving` now tries it before
  regenerating (`SaveTier::Patched { sheets }`). It works as follows:
  - Each changed sheet is indexed in one bounded pass: rows and cells in order, `r` required, depth at most 256.
  - A recalculated value is rewritten in place. The source `<f>`, attributes and formula text stay as stored.
  - Any other changed cell is written by the export's own cell writer (`CellWriter`/`cell_xml`, extracted from
    `worksheets.rs`), and new cells and rows are spliced in order.
  - New strings are appended to `sharedStrings.xml`. `uniqueCount` is set, and `count` moves by the change in
    references.
  - Every untouched zip entry is copied raw.
- Each of these falls back to regeneration, with its reason:
  - a structural change (`structure_differs` destructures `Worksheet` without `..`);
  - a style-table change (T-4561);
  - out-of-order rows or cells, or a cell without `r`;
  - a changed formula in a shared group;
  - a cell needing `metadata.xml` that the source lacks.
- Tests: `crates/rustsheet-xlsx/tests/patched_save.rs`.
  - Patching: a changed constant, a changed formula, a new cell in an existing row, a cleared cell, and a new row
    with a new string. Only `sheet1.xml` changes, or `sharedStrings.xml` as well for the new string.
  - Fallbacks: a new sheet, a new style, rows out of order, a cell without `r`, and a changed shared formula.
  - The patcher also has 4 unit tests.
- Round-trip kept rates (weighted, SPEC-0033 D8), before (T-4559's verbatim save) → after:

  | suite | failing cases | kept |
  |---|---|---|
  | roundtrip-calipers | 106 → 55 | 92.04% → 99.994% |
  | roundtrip-poi | 60 → 35 | 96.78% → 99.993% |
  | roundtrip-corpus-bench | 162 → 21 | 94.71% → 99.995% |
  | roundtrip-fixtures | 25 → 25 | 78.37% → 95.75% |

- Remaining losses:
  - Fixtures: the four external-link fixtures fall back (their cells need `metadata.xml`, T-4561) and lose their
    theme and `externalLink` parts.
  - Corpora: `calcChain`, styles and recalculated values.
- Package-invalid items on the corpora are copied from the sources, not made by the patch. They are V2 on
  in-document `#` targets in drawing relationships, and V4 on UTF-16 `customXml` parts. Both look like validator
  false positives (follow-up T-4613).

