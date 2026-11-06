---
id: T-4551
title: "ODDFYIELD near-zero and operation-order misses of the T-3988 fresh sweep"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ODDFYIELD's two misses of the T-3988 fresh sweep come within 15 digits of Excel, or the task is blocked time-boxed
(decision 0056) with the candidates recorded in financial.md.

## Context
T-3988 moved ODDFYIELD's long-first-period slope onto its own schedule (the quasi-coupon dates after settlement and the
basis' days from settlement to the first coupon; financial.md "ODDFYIELD stops short of ODDFPRICE's root"). Its fresh
sweep probes-numeric/t3988-r4-oddfyield-fresh-2 passes 118 of 120; two cells are off:
- probes-numeric/t3988-r4-oddfyield-fresh-2 S!A54
  `ODDFYIELD(DATE(2019,9,12),DATE(2036,8,31),DATE(2019,8,27),DATE(2022,8,31),0.0029,114.567,108.58,1,4)`: Excel
  -0.000530452810366501, ours -0.0005304528103664077 (1.8e-13 relative, 1e-16 absolute; the yield is near 0, so
  Excel's own value is 2.4e-8 relative from ODDFPRICE's 50-digit root). Basis 4, annual, 31 August first coupon.
  Before T-3988 the engine was 1.5e-7 off here.
- probes-numeric/t3988-r4-oddfyield-fresh-2 S!A108
  `ODDFYIELD(DATE(2006,4,3),DATE(2014,10,31),DATE(2006,1,13),DATE(2006,10,31),0.0,34.715,188.9,4,2)`: Excel
  0.2025162562959637, ours 0.202516256295963 (3.5e-15). A zero coupon, basis 2: T-3031's split exponent
  (`index + nq + DSC/E + days(Q, first coupon)/E`) passed it, T-3988's single day count (`index + nq + DFC/E`) is
  the same number rounded differently. Keeping the split with the 30/360 remainder (`(DFC - DSC)/E`) passes it but
  loses 2 cells of t3988-r4-oddfyield-month-end (117 against 119; T-3988 Outcome).
- Look for an operation order that passes A108 without losing a cell of the three ODDFYIELD sweeps
  (t3031-r4-oddfyield-long-first, t3988-r4-oddfyield-month-end, t3988-r4-oddfyield-fresh-2). Any choice made on
  these cells owes a new fresh sweep (decision 0188 R4.2).

## Acceptance criteria
- [ ] probes-numeric/t3988-r4-oddfyield-fresh-2 S!A54 and S!A108 within 15 digits of Excel with no other cell of the
  three sweeps lost, or the task blocked time-boxed with the candidates in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFPRICE's own digits (T-1374).
