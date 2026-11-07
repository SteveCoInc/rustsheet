---
id: T-4825
title: "Source-preserving export keeps an imported pivot's cache and records unchanged (roundtrip-poi 54436 regressed by T-4176)"
milestone: M16
lane: engine
status: done
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

## Outcome
- **Claim:** another worker had claimed this task, on branch `agent/engine/T-4825-poi-54436-pivot-preserve` (one
  commit, the claim only). It had no progress. The owner had me let that claim lapse and take the task over.
- **Root cause:**
  - 54436's cache is `refreshOnLoad="1"`, so the load's recalculation refreshes its pivot.
  - The refresh lays the pivot out again, and `place()` (`crates/rustsheet-engine/src/pivot/grid.rs`) always dropped
    the saved `rowItems`/`colItems` (the T-4176 rule).
  - The live pivot then differed from the pristine re-import. `export/patch.rs`'s `structure_differs` saw "the
    workbook's pivot tables changed", so the save fell to `Regenerated`, which rewrites every pivot part (cacheId
    30 to 1, the versions, the records and the `extLst` lost).
  - It was not the pivot export itself: the source-preserving tiers were never reached.
- **Fix:** a layout that changes nothing keeps the saved lines. "Nothing" means the same cache, location and
  geometry, the cells the sheet already shows (`already_shown`: same text, number or error at each place), and no
  cell to clear. Any other layout drops them, as before. SPEC-0032's export paragraph says so.
- **Tests:**
  - `workbook::tests::an_unedited_refresh_on_load_pivot_saves_its_parts_as_read` (facade): the `refreshOnLoad`
    fixture `pivot_tables.xlsx` (its GETPIVOTDATA cells removed, since they cache a placeholder 0) loads, refreshes
    and saves. Both pivots' cache definitions, records and tables, plus `workbook.xml` (the cacheIds), are the
    source's bytes. The test fails without the fix.
  - `export::pivots::tests::an_excel_saved_pivot_round_trips` now pins that an unchanged layout leaves the pivot
    as read, and still checks the engine's lines, equal to Excel's, once the saved ones are removed.
  - `a_changed_layout_drops_the_saved_lines` pins the other side: with a pivot cell edited, the layout drops them.
- **Round trip:**
  - `rustsheet-eval roundtrip --suite roundtrip-poi --filter 54436`: R0 and R1 keep 100.00%, 0 failing.
  - The re-record (`rustsheet-eval roundtrip --update-baseline --timeout 300`) removed
    `F roundtrip-poi/54436.xlsx#R0 * 296 2cd19df138a87ec5`.
- **Other #3892 lines:** I compared every round-trip baseline's current `#R0` failing lines with the baseline before
  #3892 (`a2bae4dec^`).
  - calipers, corpus-bench, fixtures and poi have identical sets, except 54436.
  - excel-saved did not exist then.
  - So 54436 was the only #3800 regression that #3892 recorded, and there is no further task to file.
- **Follow-up:** T-5046 drops the ratchet's legacy per-case path and fixes the quality-gates wording, as #4007
  promised.
