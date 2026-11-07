---
id: T-4131
title: "pivot-corpus suite: fetch Excel-saved pivot workbooks, classify fresh or stale, score import"
milestone: M16
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A fetched suite of public Excel-saved workbooks with pivots (LibreOffice sc/qa/unit/data, ClosedXML resources, plus the pivot files of calipers, poi and corpus-bench by reference), each pivot classified fresh (its cache records reproduce its saved cells) or stale, scoring import and GETPIVOTDATA now and re-render later.

## Context
oss-engines.md §5 lists about 150 files and their licences (LibreOffice MPL-2.0 and ClosedXML MIT: fetched at a pinned commit, never vendored; EPPlus Polyform-NC: excluded). Files are untrusted input. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] eval/suites/pivot-corpus/suite.toml with pinned sources and a fetcher, like calipers. (As two suites,
      `eval/suites/pivot-corpus-lo` and `eval/suites/pivot-corpus-closedxml`, since a suite has one `[fetch]`.)
- [x] Fresh/stale classification recorded per pivot; stale ones reported, not scored for render.
- [x] `make check` green; reviewers approve.

## Out of scope
- Render scoring before the PV1 renderer lands.

## Outcome
- Fetcher: `[fetch] sparse = [...]` (git sources only; validated: no `..`, no leading `/` or `-`) makes the fetch
  blob-less (`--filter=blob:none`) with `git sparse-checkout set --no-cone` before the checkout, so LibreOffice
  core costs its trees and about 9 MB of xlsx files (about 4 s). Tests in suite.rs and fetch.rs (a local
  `file://` repository).
- Suites (`gate = false`, `baseline = "cells"`, milestone M16, `require_excel_authored = true`):
  `pivot-corpus-lo` (LibreOffice b1653ced, 64 files, 87 pivots: the Excel-saved list of oss-engines.md §5) and
  `pivot-corpus-closedxml` (ClosedXML 057f5976, 19 files, 45 pivots; its 19 `TryToLoad/LO/` files are byte-identical
  copies of LibreOffice's and are left out, its ClosedXML-written outputs are not fetched; EPPlus excluded).
- Scored now (import and formula cells against Excel's caches): 4,106 cells, all passing (lo 2,515 in 9 cases,
  closedxml 1,591 in 3); 70 cases have no formula cell and are pending; no file holds GETPIVOTDATA. One case does
  not load: `pivot-corpus-closedxml/TryToLoad/SheetsWithoutRelId.xlsx` (`Missing relationship "" for worksheet`),
  owned by T-4905.
- Classification (`crates/rustsheet-eval/src/pivot_freshness.rs`; report.md "Pivot freshness", report.json
  `pivot_freshness`): lo 35 fresh, 51 stale of 86; closedxml 11 fresh, 28 stale of 39. Not classified: 6 pivots of
  files whose AppVersion 12.0000 the harness does not trust (`pivot_dark1.xlsx`, `LoadPivotTables.xlsx`,
  `TwoPivotTablesWithSingleSource/input.xlsx`) and the one of SheetsWithoutRelId. By reference: calipers 13 fresh,
  6 stale (its 13 goldens, 19 pivots); poi 1 fresh, 4 stale; corpus-bench's 4 pivot files are not classified, as
  that suite does not trust its caches. Stale reasons: 21 pivots the importer skips (no records part, a
  non-worksheet source, a calculated field), non-English captions (docs/excel-semantics/pivots.md, new entry), and
  render gaps (missing header captions, manual group labels, `""` for blank cells, sort order). Render scoring of
  fresh pivots, and a freshness test independent of the renderer, is T-4906.
- No fetched workbook has `<calculatedItems>` (nothing to note on T-4792).
