---
id: T-4491
title: "IPMT near the double range at 13 digits: IPMT(0.838240020339469,5,15,1.31635124625391E+294,4.11149200495181E+298,1)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IPMT matches Excel to 15 digits on probes-numeric/t3523-ipmt-selector-sweep Calc!C139, or the task records why the
cell is accepted (decision 0267, Tier B).

## Context
`IPMT(0.838240020339469,5,15,1.31635124625391E+294,4.11149200495181E+298,1)`: Excel 2.052481555601429E+295, the
engine 2.052481555601641E+295 from T-2243's discounted split, a 60-digit mpmath of the closed form
2.0524815556015393E+295, so both are about 5E-14 from the math on opposite sides (T-3523 Outcome). The first
wrong digit is the 13th (Tier B). It is the only cell of the sweep left off after T-4314. Candidates: an operation
order in the split that differs when `fv` dominates `pv` by 3E+4 (`A*(1-Q(x)) - fv` cancels), or type 1's `d`.
financial.md "IPMT and PPMT split Excel's discounted payment".

## Acceptance criteria
- [ ] C139 passes with a form that keeps every other IPMT cell bit-exact (`make eval-baseline`), or the task blocks
      as time-boxed (decision 0056) and the cell is accepted under decision 0267.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` cells (T-4314, T-4488).
