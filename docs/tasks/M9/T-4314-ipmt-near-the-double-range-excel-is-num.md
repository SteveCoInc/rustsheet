---
id: T-4314
title: "IPMT near the double range and at tiny rates: Excel is #NUM! where the engine has a value (319 cells of t3523-ipmt-selector-sweep)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IPMT returns `#NUM!` where Excel does near the double range and at subnormal and tiny rates, so the visibly-off cells
below pass.

## Context
T-3523's fresh sweep (probes-numeric/t3523-ipmt-selector-sweep, excel-graph 2026-11-05) found Excel `#NUM!` where the
engine returns a finite number (with T-2587's selector and without it). financial.md "IPMT near the double range and
at tiny rates" records them:
- probes-numeric/t3523-ipmt-selector-sweep.case.json, column B (200, subnormal rates written as kept-subnormal
  differences such as `(7.245817E-308-6.217028E-308)`): Calc!B1:B200
- column C (6, pv and fv 1E+290 to 9.9E+307 at ordinary rates): Calc!C44, C46, C55, C114, C179, C189; e.g. C44
  `IPMT(0.00282609064045807,30,31,2.17430063151056E+307,0,0)`, engine -4.128645778278784E+303. The other 194 column
  C cells have Excel's finite values (one, C139, 13 digits off: see below).
- column D (113, rates 1E-300 to 1E-12): Calc!D1, D3, D6:D7, D10:D11, D13:D14, D18, D20:D21, D23, D25, D28, D31,
  D34:D36, D39:D40, D44, D49, D51, D53:D54, D57:D60, D63:D65, D67, D70:D73, D76, D78:D80, D82:D89, D92, D94:D97, D99,
  D102, D104:D105, D107:D108, D110:D112, D114, D117:D120, D122:D127, D133:D135, D137:D138, D140:D143, D147, D153,
  D155:D156, D158:D161, D163:D170, D174:D180, D183, D185:D186, D191, D194, D196, D198
- C139 `IPMT(0.838240020339469,5,15,1.31635124625391E+294,4.11149200495181E+298,1)`: Excel 2.052481555601429E+295,
  engine 2.052481555601641E+295 (both about 5E-14 from the math, on opposite sides).
Every sweep cell where the engine's T-2243 discounted split declines (234) is Excel `#NUM!`, so the split's own
decline conditions are the first candidate (`discounted_split`, `payment_factor`, `discounted_balance` in
annuity.rs), then an intermediate of Excel's form overflowing or underflowing (`(pv+fv)`, `pv*(1+r)^k`,
`1-(1+r)^-n` at a tiny rate).

## Acceptance criteria
- [ ] The condition under which Excel's IPMT is `#NUM!` is identified from these cells and the passing ones of the
      same columns (a short probe where they do not decide it), and IPMT follows it.
- [ ] The cells above pass, or the ones that do not are named with their owner; no other cell regresses
      (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PPMT and PMT on the same inputs unless the same condition decides them (file a task if so).
