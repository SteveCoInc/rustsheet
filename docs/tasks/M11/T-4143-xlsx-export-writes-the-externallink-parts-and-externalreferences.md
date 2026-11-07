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
Done. A load and save keeps a workbook's links to other workbooks:
- **Export.** `export/external_links.rs` writes one `xl/externalLinks/externalLinkN.xml` part per `Workbook::external_books`
  entry, with:
  - an `externalLinkPath` relationship (`TargetMode="External"`) to the original target;
  - a content-type override;
  - the `<externalReferences>` element in source order, between `<sheets>` and `<definedNames>`.

  The sheet names, defined names and cached cells go in `CT_ExternalBook` order, typed as the import reads them back.
- **Relationship order.** The workbook's externalLink relationships come right after the sheets', as Excel numbers them,
  and the pivot caches follow (`first_external_link_id` and `first_pivot_cache_id` in workbook_xml_rels.rs).
- **Unread links.** A link the import could not read as a workbook (DDE, OLE, missing part) is written as an empty
  `<externalLink/>`, so the books after it keep their `[n]`.
- **Engine model.** `ExternalBook` gains:
  - `target`, read from the externalLink part's own relationships;
  - `data_sheets`, the sheets the cache had a `sheetData` for, since Excel writes none for some sheets
    (fixtures/external-link-uncached-lookups).
- **Import.** Cached `t="str"` text now decodes `_xHHHH_` escapes, as shared and inline strings do. This is UNVERIFIED
  and recorded in xlsx-storage.md.
- **Refactor.** `part_relationships` moved from import/pivots.rs to import/mod.rs, so one definition serves both
  callers.
- **Tests.** crates/rustsheet-xlsx/tests/external_link_export.rs covers all seven fixtures `external-link-*` cases:
  - load, save and load again gives equal books, and every cell evaluates the same;
  - one part, relationship and override per book;
  - `<externalReferences>` is placed per the schema;
  - relationship ids are distinct.

  poi/ref-56737 (1 book, 13 cells) and corpus-bench/59932 (2 books with UNC `file:///` targets, 57,172 cells) were
  checked the same way in a one-off run, because those suites are not in the repository.
- **Eval delta.** Value suites unchanged (0 fixed, 0 regressions). Round trip, failing items:
  - roundtrip-fixtures: 5,776 → 5,748;
  - roundtrip-poi: 2 files shrink (63934, link-external-workbook-b);
  - roundtrip-corpus-bench: 8 files shrink.

  roundtrip-calipers was not re-recorded. Its two pv26 goldens hit the 60 s per-case timeout here with main's binary
  too, and pass at `--timeout 300`; they hold no external link.
- **Follow-up.** T-4932 (from PR #3863's review): probe the lifts not yet probed over nested cells.
