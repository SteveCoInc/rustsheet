---
id: T-4560
title: "Source-preserving export 2: patch changed cells into the source worksheets, appending shared strings (SPEC-0034 D3.1-D3.4)"
milestone: M15
lane: engine
status: in-progress
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
- [ ] R1-style unit tests: a changed constant, a changed formula, a new cell in an existing row, a new row, and a
      cleared cell each patch only those cells, and every other part is byte-identical.
- [ ] Each fallback trigger regenerates (unit test).
- [ ] A new string is appended to `sharedStrings.xml`, and `count` and `uniqueCount` are updated.
- [ ] The round-trip baselines are rerecorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New styles and `metadata.xml` (T-4561).
