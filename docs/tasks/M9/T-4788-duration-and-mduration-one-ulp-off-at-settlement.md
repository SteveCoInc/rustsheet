---
id: T-4788
title: "DURATION and MDURATION one ulp off at settlement 61 (1900-03-01) on fin-coupon-walk-bounds-c, and two gen-m9 YIELD one-ulp cells no task names (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The six bond cells below match Excel to the bit. Failing that, the Outcome lists the forms tried with their bit-exact
counts over every DURATION, MDURATION and YIELD oracle cell (decision 0056 time box).

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main c18fafcce, excel-graph). An ownership check ran over every
  failing Excel cell (`scripts/exit_owners.py`'s `owned` rule against the open task files). It found these six cells
  named by no open task. None is visibly off, so this is Phase 1b work.
- DURATION and MDURATION, probes/fin-coupon-walk-bounds-c (excel-graph, T-1844). Quarterly bonds, 5% coupon, 6% yield,
  basis 0, settlement 61 (1900-03-01):

  | cell | formula | Excel | ours |
  |---|---|---|---|
  | I13 | `DURATION(61,36585,0.05,0.06,4)` | 16.91311178460704 | 16.913111784607043 |
  | I14 | `MDURATION(61,36585,0.05,0.06,4)` | 16.663164319809894 | 16.663164319809898 |
  | K13 | `DURATION(61,36950,0.05,0.06,4)` | 16.91364482311201 | 16.913644823112012 |
  | K14 | `MDURATION(61,36950,0.05,0.06,4)` | 16.663689480898533 | 16.663689480898537 |

  - The same bonds from settlement 59 and 60 (G13:H14, J13:J14) are bit-exact, so the miss depends on where
    settlement falls against the February 1900 coupon date. COUPPCD is serial 59 there (financial.md "Coupon schedules
    at the calendar's edges"), which gives A = 1 and DSC/E = 89/90 on basis 0. At 59 and 60, A is 0 and -1.
  - A triage toy in Python doubles copies the engine's flows (`fn_duration`, financial_bonds.rs: N coupons at `k + DSC/E`
    and the redemption alone, `(t·cf)/v^t`, with `**` standing in for `^`). It gives ours, 16.913111784607043 for I13
    and 16.913644823112012 for K13, whether the redemption sits at `DSC/E + N - 1` or at `(N - 1) + DSC/E`. So the PRICE redemption order T-2788 found (financial.md, T-2788 paragraph; "DURATION's own
    cash-flow sum (`add_cash_flow`) still takes `DSC/E + n - 1`") does not explain I13 on its own.
  - The next candidates are the discount factor (`^` against a running product, T-3144's kernel question), the sum's
    order and the final division.
  - T-3723 and T-1374 own DURATION's multi-coupon last digits on other cases; check their candidate tables first.
- YIELD, gen-m9 (2 cells, ulp1):
  - gen-m9/9a1fa5366bc50918 Calc!A231 `YIELD(39148,45562,Data!D3,81,104,1)` (Data!D3 = 7, annual, price 81): Excel
    3.4746998470918595, ours 3.474699847091859.
  - gen-m9/bc04c1f478c9896c Calc!A23 `YIELD(38063,45986,BITLSHIFT(908,-8),102,109,2)` (rate 3.546875, semiannual):
    Excel 2.56744131580262, ours 2.5674413158026197.
  - The first is a deep-discount bond, like T-4758's 1-to-2-ulp deep-discount cells. Its findings come first. Neither
    T-4758 nor T-3620 names these two cells.
- Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (DURATION's `add_cash_flow`),
  financial_bonds/yield_solver.rs (`excel_yield_newton`), financial_bonds/coupon_schedule.rs.

Cells this task owns:
- probes/fin-coupon-walk-bounds-c.case.json (4): Sheet1!I13, I14, K13, K14
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A231
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A23

## Acceptance criteria
- [ ] The six known-failure lines leave eval/baseline/probes.txt and eval/baseline/gen-m9.txt (`make eval-baseline`)
      with no new line in any baseline. Otherwise the Outcome lists each form tried with its bit-exact count over
      every DURATION, MDURATION and YIELD oracle cell.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7) for any fitted form, recorded in the Outcome:
  - hold-out counts: a hold-out split fixed before scoring, with its bit-exact count;
  - a fresh sweep drawn after the form is frozen, with its count;
  - the Gate A envelope result for DURATION, MDURATION and YIELD;
  - a `FITTED` tag in the code with the range it was fitted on;
  - the reproducibility floor;
  - no keep-matched branch.
- [ ] Needs an oracle sweep before any fit. The cached cells have one settlement-61 pair per maturity and two YIELD
      points. File a research sweep task (settlements 59 to 70 against month-end maturities, quarterly and semiannual,
      basis 0 to 4) and depend on it if no open one covers it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's deep-discount cells and P!B13 (T-4758); YIELD's `#NUM!` cells (T-4523); the `^` kernel (T-3144).
- DURATION and MDURATION cells of T-3722, T-3723, T-2972, T-2973 and T-1374.

## Deferred (decision 0114)
`defer: phase-1b`: every owned cell is within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove
the key only when the steward records the Phase 1 exit, or when an eval shows an owned cell visibly off (then give it
a `campaign:` stage).
