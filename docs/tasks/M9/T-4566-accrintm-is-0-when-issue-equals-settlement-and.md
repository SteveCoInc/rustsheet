---
id: T-4566
title: "ACCRINTM is 0 when issue equals settlement; ACCRINT's basis-2 issue period and its 0 at the calendar's start"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ACCRINTM returns Excel's 0, not `#NUM!`, when the issue equals the settlement. ACCRINT counts the issue's period on
basis 2 as Excel does (US 30/360 moving both dates, as on basis 0). ACCRINT's result with calc_method TRUE,
a settlement after first_interest, and a coupon date on the walk back before 1900-01-01 is settled: Excel for the web
has given both 0 and `#NUM!` there, and the engine follows what a fresh fill shows.

## Context
- probes/accrint-zero-results B1:B40 (excel-graph, 2026-11-06, T-4114): `ACCRINTM(d,d,0.05,1000,b)` is 0 for every
  d in 0, 1, 59, 60, 61, 366, 45000 and 2958465 and every basis. The engine returns `#NUM!` (`accrintm_of` in
  crates/rustsheet-engine/src/functions/financial/discount.rs checks `issue_serial >= settlement_serial`). A reversed
  pair stays `#NUM!` (accuracy/ACCRINTM/s1 A136 `ACCRINTM(45100,44968,..)`). Equal dates with a rate or par at most 0
  are unprobed.
- Gate A: accuracy/ACCRINTM/s1 A115 `ACCRINTM(59,59,..)` and A134 `ACCRINTM(60,60,..)` are carried in
  eval/accuracy/known-failures.tsv under this task.
- probes/accrint-zero-results A1:A9, A12, A14, A16:A19: `ACCRINT(61,91,45000,0.05,1000,4,b,TRUE)` and its neighbours
  (issue 1900-03-01 to 1900-03-30, first_interest 1900-03-30, 1900-03-31 or 1900-06-30, settlement after it) are 0 in
  Excel on 2026-11-06 (quarterly on bases 0-4, semiannual on bases 0 and 1, annual on basis 0); the engine gives `#NUM!`. With calc_method FALSE (A11) or a settlement
  before first_interest (A10) both are `#NUM!`. financial.md "ACCRINT at the calendar's start" records `#NUM!` for the
  same shape from the 2026-10-24 to 10-26 fills (and a 0 on 2026-10-17), so re-fill those probes before changing
  `compute_accrint`.
- probes/accrint-zero-results C15, C16 and accuracy/ACCRINT/s1 A21 (carried in known-failures.tsv under this task):
  `ACCRINT(49107,49552,49782,0.1,1000,2,2,FALSE)` is 137.22222222222223 in Excel, the engine 137.09944751381215. Excel's
  basis-2 issue-period length is US 30/360 moving both dates (C3, C4, C9, C10 tell it from 360/frequency); the engine's
  `stub_denominator` (crates/rustsheet-engine/src/functions/financial/accrint.rs) uses plain US 30/360 on basis 2.
  tools/src/rustsheet_tools/accuracy/defs/accrint.py `_issue_period` already follows Excel.
- Semantics: financial.md "ACCRINT, ACCRINTM, PRICEMAT and YIELDMAT: amount domains (T-4114)".

## Acceptance criteria
- [ ] ACCRINTM with equal dates and a positive rate and par is 0; probes/accrint-zero-results B1:B40 and
      accuracy/ACCRINTM/s1 A115, A134 pass, and their known-failures.tsv lines are removed.
- [ ] ACCRINT's basis-2 issue period is US 30/360 moving both dates; probes/accrint-zero-results C15, C16 and
      accuracy/ACCRINT/s1 A21 pass and A21's known-failures.tsv line is removed.
- [ ] A re-fill of probes/accrint-zero-results and of the "ACCRINT at the calendar's start" probes decides ACCRINT's
      result there; financial.md and, if it is stable, the engine and defs/accrint.py follow it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ACCRINT's unprobed points (T-4522).

## Outcome
2026-11-06.
- **ACCRINTM** (`accrintm_of`, functions/financial/discount.rs): an issue equal to the settlement is 0 once the rate and
  par are positive; a reversed pair stays `#NUM!`. probes/accrint-zero-results B1:B40 and accuracy/ACCRINTM/s1 A115,
  A134 pass.
- **ACCRINT basis 2** (`stub_denominator`, functions/financial/accrint.rs): the issue's period length is US 30/360
  moving both dates (the existing `days_30_360_us_both_ymd`, as on basis 0). probes/accrint-zero-results C15, C16 and
  accuracy/ACCRINT/s1 A21 pass. defs/accrint.py already did this; its docstring no longer says the engine departs.
- **ACCRINT at the calendar's start: not changed.** Re-filled probes/accrint-zero-results, fin-accrint-1900-calendar,
  fin-coupon-walk-bounds-b, -c, accrint-calendar-start-1900, -1904 and accrint-eom-1900 twice (`fill --force`, oracle
  cache emptied between; 15:08 and 15:10 UTC). The two fills agreed with each other, and 140 cells differed from the
  committed 2026-10-20 to 10-27 sidecars: a walk back reaching a period start before 1900-01-01 is now 0 with
  calc_method TRUE or FALSE, and basis 1's regular-period cut and most of the 1904 cut are gone. The multi-cell
  answer moved from `#NUM!` (10-20 to 10-27) to 0 (11-06), and fin-coupon-walk-bounds-b A9, the only cell filled on
  10-17, went 0, `#NUM!`, 0; the answer flips over time, so the engine, defs and committed sidecars
  stay as they were and accrint-zero-results A1:A9, A12, A14, A16:A19 stay known failures (financial.md "ACCRINT at
  the calendar's start", bullet "Re-fill of 2026-11-06"). Follow-up T-4631 re-fills after 2026-11-20 and decides.
- Unit tests: `fn_accrintm_issue_equals_settlement_is_zero` (replaces the `#NUM!` test for equal dates),
  `accrint_basis_2_issue_period_moves_both_dates`.
- **Eval delta:** probes known failures 7525 -> 7483 (fixed 42: accrint-zero-results B1:B40, C15, C16), no
  regressions; Gate A 0 failing, its three T-4566 lines removed from eval/accuracy/known-failures.tsv.
