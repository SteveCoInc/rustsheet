---
id: T-4143
title: "xlsx export writes the externalLink parts and externalReferences that [n] references need"
milestone: M11
lane: engine
status: todo
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
- [ ] Load, then save, then load again of every fixtures `external-link-*` case, poi ref-56737 and corpus-bench 59932
      gives the same external books (target, sheet names, defined names, cached values), and the same values for
      cells that read them.
- [ ] The saved package has one externalLink part per book, with an External-mode relationship to the original
      target, a content-type override, and `<externalReferences>` in source order.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Formula text (T-4140).
- Refreshing links (no I/O, directive 5).
