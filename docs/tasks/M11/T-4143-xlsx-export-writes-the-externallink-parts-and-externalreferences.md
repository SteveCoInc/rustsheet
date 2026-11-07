---
id: T-4143
title: "xlsx export writes the externalLink parts and externalReferences that [n] references need"
milestone: M11
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4140]
---

## Goal
A workbook with links to other workbooks keeps them when saved: the `xl/externalLinks/externalLinkN.xml` parts, their
relationships (the target path) and `<externalReferences>` in workbook.xml, written from the external books the
importer already reads, so `[n]` references in saved formulas resolve in Excel with their cached values.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
All 70 corpus files with external links lose them (poi 10, calipers 4, corpus-bench 46, fixtures 10). The importer reads
sheet names, defined names and cached cells into `Workbook::external_books`
(`crates/rustsheet-xlsx/src/import/external_links.rs`, `crates/rustsheet-engine/src/external_links.rs`). The export
writes no part (`crates/rustsheet-xlsx/src/export/mod.rs`). This task models the parts, which is M11 cell content. The
generic preservation of unmodelled parts is M15 (decision 0275).

## Acceptance criteria
- [x] Load, then save, then load again of every fixtures `external-link-*` case, poi ref-56737 and corpus-bench 59932
      gives the same external books (target, sheet names, defined names, cached values), and the same values for
      cells that read them.
- [x] The saved package has one externalLink part per book, with an External-mode relationship to the original
      target, a content-type override, and `<externalReferences>` in source order.
- [x] `make check` green; reviewers approve.

## Out of scope
- Formula text (T-4140).
- Refreshing links (no I/O, directive 5).

## Outcome
The export writes one `xl/externalLinks/externalLinkN.xml` per external book, in `[n]` order
(`crates/rustsheet-xlsx/src/export/external_links.rs`). Each part has its `externalLinkPath` relationship
(`TargetMode="External"`) to the book's original target, a content-type override, a workbook relationship and an
`<externalReference>`, placed between `<sheets>` and `<definedNames>`. To save the target, `ExternalBook` gains
`target`, read from the part's own relationships. It also gains `cached_sheets`, the sheets the cache had a
`sheetData` for: Excel leaves some sheets out (external-link-uncached-lookups has none for sheet 3), and the
round-trip gate flagged an export that wrote one per sheet name. Neither field changes evaluation.

Tests: `crates/rustsheet-xlsx/tests/external_link_export.rs` covers the seven fixtures `external-link-*`: a reload
gives the same books (target, sheet names, names, cells) and the same value in every cell, and a second save writes
the same part. A second test checks the package wiring (target `56737.xlsx`, override, rId, element order), and a
third checks that a workbook with no links writes no part. Unit tests in `export/external_links.rs` cover the part
XML.

Eval delta: the formula suites are unchanged (no F line moves). Round trip (`make eval-baseline`):
- roundtrip-fixtures: 28 failing items fixed (5776 -> 5748), R1 kept 93.65% -> 94.57%.
- roundtrip-corpus-bench: 8 files' failing items 6200 -> 4082.
- roundtrip-poi: 63934 and link-external-workbook-b 337 -> 289.

roundtrip-calipers is not re-recorded. Under the default 60 s, pv26_002 and pv26_2_002 time out on this 4-CPU box.
Alone, pv26_002 takes 155 s and is fully kept; it has no external link. The nightly gate runs calipers at
`--timeout 300`. A full local `--check --timeout 300` was OOM-killed, so the nightly run is the check for the rest of
calipers.

The acceptance cases poi ref-56737 (1 book, `56737.xlsx`) and corpus-bench 59932 (2 books, `file:///\\NAS...`
UNC targets) were checked locally: a reload gives equal books and the same value in every cell. Both are fetched,
not vendored, so the committed test reads only the fixtures. Both are already clean in the round-trip baselines
(`C` lines with no `F`), before and after this change.
