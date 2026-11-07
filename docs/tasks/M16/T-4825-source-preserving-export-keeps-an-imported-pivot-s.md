---
id: T-4825
title: "Source-preserving export keeps an imported pivot's cache and records unchanged (roundtrip-poi 54436 regressed by T-4176)"
milestone: M16
lane: engine
status: todo
priority: P0
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

Raised to P0 by the 2026-11-07 steward PR (#3872), which bisected the same regression (e7f6fbb0b T-4363 keeps 100.00%,
cd0c355ad T-4176 keeps 78.79%) and makes this task its one owner (T-4829 closed as a duplicate; the PR-gate question is
T-4867's). New fact: #3892 (T-4514, a2bae4dec, round-trip R1) re-recorded `eval/baseline/roundtrip-poi.txt` with 332
new `F` lines, among them `F	roundtrip-poi/54436.xlsx	* 296 2cd19df138a87ec5` (and `C	roundtrip-poi/54436.xlsx	1990`,
995 before it), citing no decision for that line. So `roundtrip --check` now passes on `main` with the regression
recorded, the nightly gate no longer reports it, and a passing check is no evidence of a fix.

## Acceptance criteria
- [ ] With the poi corpus fetched, `rustsheet-eval roundtrip --suite roundtrip-poi --filter 54436` keeps 100.00% with
      0 failing, and the PR removes the `F	roundtrip-poi/54436.xlsx` line from `eval/baseline/roundtrip-poi.txt` by
      re-recording the fix (never by hand, no `--allow-regressions`).
- [ ] The PR checks whether any other `F` line #3892 added is a regression from #3800 rather than a new R1 finding,
      and files a task for each one that is.
- [ ] A unit or fixture test pins that an unedited imported pivot's cache, records and cacheId survive export.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Exporting pivots the API created or edited (T-4176's scope); only the unedited-import path.
