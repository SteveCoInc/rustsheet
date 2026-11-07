---
id: T-4798
title: "Round-trip regression: poi/54436.xlsx keeps 78.79% after T-4176's pivot export rewrites the package (was 100.00%)"
milestone: M16
lane: engine
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Loading and saving poi/54436.xlsx keeps what it kept before T-4176: `rustsheet-eval roundtrip --check --suite
roundtrip-poi` passes against `eval/baseline/roundtrip-poi.txt` without re-recording the case.

## Context
Found by the steward run of 2026-11-07 ("Eval regression found later", docs/process/agent-workflow.md). With the poi
corpus fetched, `rustsheet-eval roundtrip --check` on `main` at 42d9bdaa reports
`REGRESSION roundtrip-poi/54436.xlsx * 296 2cd19df138a87ec5`. The baseline has no failing line for the case (checked
995).

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
UTC, before #3800, so the next Nightly eval should go red on this case.

## Acceptance criteria
- [ ] `rustsheet-eval roundtrip --check --timeout 300 --suite roundtrip-poi` passes on the PR's head with the poi
      corpus fetched. 54436.xlsx is back to 100.00% kept, with no `--update-baseline --allow-regressions`.
- [ ] A regression test in crates/rustsheet-xlsx pins the cause: loading and saving an unedited workbook with a pivot
      keeps the source package's parts as SPEC-0034 says.
- [ ] T-4176's pivot export still passes its own tests and probes.
- [ ] The PR says whether SPEC-0035's PR gate covers round-trip poi, and files a task if it does not.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Round-trip losses already in the baseline for other cases.
