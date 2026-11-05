---
id: T-4314
title: "IPMT near the double range: Excel is #NUM! where an intermediate overflows (5 cells of t3523-ipmt-selector-sweep)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IPMT returns `#NUM!` where Excel does near the double range, so the five visibly-off cells below pass.

## Context
T-3523's fresh sweep (probes-numeric/t3523-ipmt-selector-sweep, excel-graph 2026-11-05) put pv and fv between 1E+290
and 9.9E+307 at ordinary rates. Excel is `#NUM!` on these five; the engine returns a finite number (with T-2587's
selector and without it):
- probes-numeric/t3523-ipmt-selector-sweep.case.json (5): Calc!C33, C36, C45, C99, C180
- C33 `IPMT(0.00286322465373438,2,13,1.40355762329685E+305,7.40149559572831E+307,0)`: engine 1.5651981778187516E+304
- C36 `IPMT(0.0077709833702648,2,3,2.59956670396635E+303,1.22820359711015E+307,0)`: engine 3.1555025483483887E+304
- C45 `IPMT(0.00799876951004341,3,5,2.71531320084367E+307,1.09784064008143E+306,1)`: engine -1.2686638277892457E+305
- C99 `IPMT(-0.00370928755867435,15,16,4.70938571164442E+307,0,0)`: engine 2.127180666245031E+304
- C180 `IPMT(-0.0140196032744638,2,6,2.58281603156391E+298,1.670798343956E+307,0)`: engine -4.043072804768575E+304
The other 195 cells of column C pass, including finite results of the same size, so the error is an intermediate
of Excel's own form overflowing (candidates: `(pv+fv)` or `pv·(1+r)^k` before the rate multiplies; T-2243's
`A = (pv+fv)/(1-Q)`). financial.md "IPMT near the double range" records the cells.

## Acceptance criteria
- [ ] The intermediate whose overflow makes Excel `#NUM!` is identified from these cells and column C's passing
      ones (a short probe of pv+fv just below and above 1.797E+308 if they do not decide it), and IPMT follows it.
- [ ] The five cells pass; no other cell regresses (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PPMT and PMT near the double range unless the same intermediate decides them (file a task if so).
