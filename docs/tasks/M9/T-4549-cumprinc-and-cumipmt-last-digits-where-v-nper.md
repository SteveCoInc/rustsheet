---
id: T-4549
title: "CUMPRINC and CUMIPMT last digits where v^nper underflows: the 130 probes/fin-cumprinc-underflow-fresh and -type1 cells T-3141 does not name (deferred)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3141]
defer: phase-1b
---

## Goal
After T-3141 lands its underflowed-principal core, the CUMPRINC and CUMIPMT cells below are Excel's to the bit, or
every form tried is recorded in docs/excel-semantics/financial.md. These cells are T-3141's post-freeze sweep, and
no open task names them.

## Context
- Gap triage 2026-11-06 (`make eval` on main 25779368). Every visibly-off cell has an open owner. Of the 308,260
  failing cells, 196 have no open task. 66 of those are probes-numeric/t3367-nper-tiny-rate cells, where the engine
  returns the math (decision 0188 R3), so they pass the exit measure. The other 130 are the cells below. All of them
  are within 15 digits: 77 within 1 ulp, 40 within 4 ulp, 13 `digits15`. So this task is `defer: phase-1b` (decision
  0114).
- The two cases are T-3141's sweep after the freeze of the underflowed-principal core
  `C = pv·((1-v^m)·v^(nper-end+1))` (type 1 `-C`, type 0 `-(C·(1+r))`), salvaged from draft PR #3105 by T-4385. Their
  description gives five rates (0.02, 0.0004, 0.15, 0.07, 0.035), each with its own nper and pv, in four columns each
  (CUMPRINC type 0 and 1, CUMIPMT type 0 and 1: A-D, E-H, I-L, M-P, Q-T). T-3141 owns the CUMIPMT cells it lists
  (fresh C1, C2, C6, D1, ...), and T-4059 owns type1 C7:C9 and G7:G9 and fresh I10 and K10. The cells here are the
  rest:
  - probes/fin-cumprinc-underflow-fresh.case.json (90): Sheet1!A1, A2, A5, A6, B1, B2, B4, B5, B6, B8, C5, D4, D5,
    E1, E2, E3, E4, E6, E8, E9, F1, F2, F3, F4, F8, F9, F10, I2, I4, I5, I7, J1, J3, J4, J5, J6, K2, K4, K5, K7, L1,
    L3, L4, L5, L6, M1, M2, M3, M4, M6, M7, M8, M9, M10, N1, N2, N3, N4, N5, N6, N7, N8, N9, O1, O2, O3, O4, O6, O8,
    O9, O10, P1, P2, P3, P4, P5, P6, P8, P9, Q2, Q4, Q10, R2, R4, R8, R9, S4, S10, T4, T8
  - probes/fin-cumprinc-underflow-type1.case.json (40): Sheet1!A2, A3, A4, A5, A9, B1, B2, B4, B7, B9, C2, C3, C6,
    D2, D4, D6, D7, D8, D9, E2, E3, E4, E6, E8, E9, F1, F2, F4, F6, F7, F9, G1, G2, G4, G5, G6, H3, H4, H7, H8
- Examples (Excel, then ours): fresh A1 `CUMPRINC(0.02,1E5,987.65,1E5-0,1E5,0)` -19.365686274509805,
  -19.3656862745098 (1 ulp); fresh E9 `CUMPRINC(0.0004,3E9,3300000,3E9-4,3E9-3,0)` -2635.253275357145,
  -2635.253275357146; fresh C5 `CUMIPMT(0.02,1E5,987.65,1E5-9,1E5,0)` -20.09699837169714, -20.09699837169711
  (`digits15`: CUMIPMT amplifies the principal's last bits); type1 E8 `CUMPRINC(0.05,5000,12345.6,5000-3,5000-2,1)`
  -991.4928156771833, -991.4928156771836.
- First re-score after T-3141 merges: its kernel may fix many of these. Re-list what is left before you start.
- Needs an oracle sweep. These cells are themselves T-3141's fresh sweep, so they cannot also be the hold-out for a
  form chosen on them. T-4059's `r4-T-3141` Gate A batches are the other evidence. Any fit beyond those needs a new
  research-lane sweep task (`make new-task`), and this task depends on it. Do not fit blind. Decision 0056 applies:
  two attempts, then time-box.

## Acceptance criteria
- [ ] The cells above that are still failing after T-3141 pass (`make eval-baseline`, no new known failures), or each
      form tried is recorded in financial.md with the cells it fixes and breaks.
- [ ] Any fitted form meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188): hold-out counts
      reported (cells fixed and broken on points not used to choose the form); a fresh sweep filled after the form was
      frozen; the Gate A envelope result of CUMPRINC's and CUMIPMT's tables (`rustsheet-eval accuracy`, no new
      failing point); a `FITTED` tag in the code with its measured range; the reproducibility floor; and no
      keep-matched branch that special-cases the cells already passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3141's own cells (probes/fin-cumprinc-near-1e15 and the fresh CUMIPMT cells it lists) and T-4059's cells.
- Ranges where `v^nper` does not underflow (T-3216, T-4390).
