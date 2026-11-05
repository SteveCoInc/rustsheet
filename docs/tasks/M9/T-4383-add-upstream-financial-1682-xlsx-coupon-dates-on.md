---
id: T-4383
title: "Add upstream FINANCIAL/1682.xlsx (coupon dates on the 29th and 30th) to calc_tests"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The Excel-authored workbook upstream IronCalc added on 2026-11-05, `FINANCIAL/1682.xlsx` (ironcalc `d78f5668`,
the test workbook of the coupon-date fix `67f7856b`), is in `crates/rustsheet-xlsx/tests/calc_tests` with every
cell passing.

## Context
Upstream triage, steward 2026-11-05 (second run). The code fix in `67f7856b` needs no port: our
`coupon_period` in `functions/financial/financial_bonds/coupon_schedule.rs` already moves the maturity back a whole
number of periods with the day clamped to the month, and COUPDAYS on basis 1 already walks the chained schedule
(docs/excel-semantics/financial.md, "Coupon dates keep the maturity's day", VERIFIED on excel-graph, T-1489, T-1493,
T-2486, T-2640).

The workbook may still add Excel-computed cells we do not have. Recover it with
`git -C target/upstream-ironcalc show d78f5668:xlsx/tests/calc_tests/FINANCIAL/1682.xlsx`
(`scripts/upstream-log.sh` keeps the clone). First check it is Excel-authored (the suite sets
`require_excel_authored`; `rustsheet-eval snapshot` shows the application), and leave it out with a note here if it
is not. `build.rs` makes a strict test per workbook, so add it only when all its cells pass (precedent T-3857,
T-1517, T-0965). Check cells against the workbook's own cached values, never against the upstream commit message
(untrusted). When a cell fails, fix it here if S-sized, or else file a follow-up naming the cells and leave the
workbook out. Record any new quirk in `docs/excel-semantics/financial.md`.

## Acceptance criteria
- [ ] The workbook passes in full and is in calc_tests (`rustsheet-eval show` on it shows every cell passing;
      baseline updated, 0 regressions), or it is left out with a filed follow-up task naming its failing cells (or
      with a note that it is not Excel-authored).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting upstream's code change (ours already keeps the maturity's day).
