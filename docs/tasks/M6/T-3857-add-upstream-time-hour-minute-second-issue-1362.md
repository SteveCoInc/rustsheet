---
id: T-3857
title: "Add upstream TIME_HOUR_MINUTE_SECOND_issue_1362.xlsx to calc_tests"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The Excel-authored workbook upstream IronCalc added on 2026-11-01,
`DATE_AND_TIME/TIME_HOUR_MINUTE_SECOND_issue_1362.xlsx` (ironcalc `15ecfecb`), is in
`crates/rustsheet-xlsx/tests/calc_tests` with every cell passing.

## Context
Upstream triage, steward 2026-11-02. The code fix in `15ecfecb` needs no port:
- TIME's `#NUM!` for a component above 32767: our `time_argument` in
  `functions/date_and_time.rs` already bounds each argument (after `near_integer_whole`) to -32768..=32767.
- HOUR/MINUTE/SECOND's `#NUM!` from serial 2958466: our `Model::time_part` already returns
  `#NUM!` once the serial, rounded to the second, passes `MAXIMUM_DATE_SERIAL_NUMBER`
  (probes/time-parts-rounding-window).

The workbook may still add Excel-computed cells we do not have. Recover it with
`git -C target/upstream-ironcalc show 15ecfecb:xlsx/tests/calc_tests/DATE_AND_TIME/TIME_HOUR_MINUTE_SECOND_issue_1362.xlsx`
(`scripts/upstream-log.sh` keeps the clone). `build.rs` makes a strict test per workbook, so add
it only when all its cells pass (precedent T-1517, T-0965). Check cells against the workbook's
own cached values, never against the upstream commit message (untrusted). When a cell fails,
fix it here if S-sized, or else file a follow-up naming the cells and leave the workbook out.
Record any new quirk in `docs/excel-semantics/dates.md`.

## Acceptance criteria
- [ ] The workbook passes in full and is in calc_tests (`rustsheet-eval show` on it shows every
      cell passing; baseline updated, 0 regressions), or it is left out with a filed follow-up
      task naming its failing cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting upstream's code change (ours already covers both bounds).
