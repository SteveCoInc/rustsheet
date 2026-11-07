---
id: T-4825
title: "Source-preserving export keeps an imported pivot's cache and records unchanged (roundtrip-poi 54436 regressed by T-4176)"
milestone: M16
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Exporting the POI workbook `54436.xlsx` (one imported pivot, no edits) round-trips as it did before T-4176: `roundtrip-poi/54436.xlsx` passes again with all 995 checked items kept, and `rustsheet-eval roundtrip --check` is clean wherever the POI cache is fetched.

## Context
T-4176 (#3800, `cd0c355ad`) exports pivot caches, records and definitions. Since that commit the round-trip of
`roundtrip-poi/54436.xlsx` fails (baseline line `* 296 2cd19df138a87ec5`; it passed on `cd0c355ad^`,
bisected while working on T-4598). The failing R0 items show the pivot parts rewritten instead of
preserved: `pivotCache@cacheId` 30 became 1, `pivotCacheDefinition@createdVersion` and `@refreshedVersion`
4 became 6, `@refreshedBy`/`@refreshedDate`, the definition's `extLst` and every `pivotCacheRecords/r/*`
value are lost, and `workbook/fileVersion` and `workbookPr` are lost too (the package no longer takes the
source-preserving patch path, SPEC-0034).

PR CI does not catch it: `roundtrip-poi` reads the POI fetched cache, which only the nightly fetches
(`.github/workflows/nightly.yml`), so every local `make eval-baseline` / `make check` with the cache
fetched now refuses this regression. Code paths: `crates/rustsheet-xlsx/src/export/pivots/**` and the
patch-or-rewrite decision in the xlsx export; reproduce with
`rustsheet-eval roundtrip --check --suite roundtrip-poi`.

## Acceptance criteria
- [ ] `roundtrip-poi/54436.xlsx` passes again (no `F` line for it in `eval/baseline/roundtrip-poi.txt`).
- [ ] A unit or fixture test pins that an unedited imported pivot's cache, records and cacheId survive export.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Exporting pivots the API created or edited (T-4176's scope); only the unedited-import path.
