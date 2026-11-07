---
id: T-4798
title: "Round-trip regression: poi/54436.xlsx keeps 78.79% after T-4176's pivot export rewrites the package (was 100.00%)"
milestone: M16
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Loading and saving poi/54436.xlsx keeps what it kept before T-4176 (100.00%, no failing line), and the failing line
that #3892 recorded for it leaves `eval/baseline/roundtrip-poi.txt`.

## Context
Found by the steward run of 2026-11-07 ("Eval regression found later", docs/process/agent-workflow.md). With the poi
corpus fetched, `rustsheet-eval roundtrip --check` on `main` at 42d9bdaa reports
`REGRESSION roundtrip-poi/54436.xlsx * 296 2cd19df138a87ec5`. The baseline has no failing line for the case (checked
995).

**Update (2026-11-07, after the steward PR caught up with `main`):** #3892 (T-4514, a2bae4dec, round-trip R1) re-recorded
`eval/baseline/roundtrip-poi.txt` with 332 new `F` lines, among them `F	roundtrip-poi/54436.xlsx	* 296 2cd19df138a87ec5`
(and `C	roundtrip-poi/54436.xlsx	1990`). So `roundtrip --check` now passes on `main` with the regression recorded as a
known failure, the nightly gate will not go red on it, and a passing check is no evidence of a fix. That commit cites
no decision for this line; the PR for this task says so.

Bisected with `rustsheet-eval roundtrip --suite roundtrip-poi --filter 54436`:
- e7f6fbb0b (T-4363, the parent of #3800): 995 items, 100.00% kept, 0 failing;
- cd0c355ad (T-4176, #3800, "Export pivot caches, records and definitions to xlsx"): 78.79% kept, 1 failing.

The workbook holds one pivot table on Sheet1. After T-4176 the save no longer keeps the source package: the R0 items
show it regenerated, for example core and extended properties (`Application` "Microsoft Excel" became "IronCalc
Sheets"), the theme's fmtScheme and fonts, `workbookPr`, `fileVersion`, styles' `extLst` and `tableStyles`, the sheet's
`pageMargins`, `pageSetup` and printer settings, the pivot cache records, `refreshedBy`/`refreshedDate`, and the
cacheId 30 became 1. Judgment: writing the pivot parts appears to mark the workbook changed, so the source-preserving
export of SPEC-0034 falls back to a full write. Confirm before fixing.

PR CI did not catch it. #3800 merged at 01:17 UTC, after SPEC-0035's fetched-suite gate (#3779, 2026-11-06 23:56 UTC).
Check whether that gate runs `roundtrip --check` for poi on PRs that touch crates/rustsheet-xlsx. If it does not, say
so in the PR and file a follow-up. The nightly corpus round-trip gate (decision 0323) last passed at 2026-11-06 14:16
UTC, before #3800; because of #3892's re-record (above) it will not go red on this case.

## Acceptance criteria
- [ ] With the poi corpus fetched, `rustsheet-eval roundtrip --suite roundtrip-poi --filter 54436` keeps 100.00%
      with 0 failing on the PR's head, and the PR removes the `F	roundtrip-poi/54436.xlsx` line from
      `eval/baseline/roundtrip-poi.txt` by re-recording the fix (never by hand, no `--allow-regressions`);
      `roundtrip --check --timeout 300 --suite roundtrip-poi` then passes.
- [ ] The PR checks whether any other `F` line #3892 added is a regression from #3800 rather than a new R1 finding,
      and files a task for each one that is.
- [ ] A regression test in crates/rustsheet-xlsx pins the cause: loading and saving an unedited workbook with a pivot
      keeps the source package's parts as SPEC-0034 says.
- [ ] T-4176's pivot export still passes its own tests and probes.
- [ ] The PR says whether SPEC-0035's PR gate covers round-trip poi, and files a task if it does not.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Round-trip losses already in the baseline for other cases.

## Outcome
Duplicate of T-4825, which already owned this regression on `main` (as did T-4829, also closed, and T-4867, which keeps
the PR-gate question). Filed by the 2026-11-07 steward PR (#3872); its new facts (the bisect, and #3892 recording the
regression as an `F` line) and its acceptance criteria moved to T-4825, now P0. No change here.
