---
id: T-4524
title: "CUMPRINC and CUMIPMT last digits on T-3141's underflow sweeps: 130 fin-cumprinc-underflow-fresh and -type1 cells within 15 digits that no task names (Phase 1b)"
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
Each listed cell of T-3141's two underflow sweeps returns Excel's double, or the Outcome lists it with Excel, ours,
the 60-digit reference (fit lab `kernels/cumulative.py`) and the reason Excel's last bits cannot be derived. The cells
are CUMPRINC where `v^nper` underflows or `1 - v^nper` rounds to 1, and the CUMIPMT built on it. Every one is already
within half a unit of the 15th significant digit, so this is Phase 1b work only (decision 0114).

## Context
- Gap triage 2026-11-06 (opus refill, `make eval` on origin/main a6cb9afed). Ownership was checked with
  `scripts/exit_owners.py`'s `owned` over every failing cell, not only the visibly-off ones. These 130 cells were the
  largest failing group no open task owns. `make unowned-off ALL=1` was clean, because none of them is visibly off.
- The cases: probes/fin-cumprinc-underflow-fresh and probes/fin-cumprinc-underflow-type1. T-4385 salvaged them from
  draft PR #3105, and T-3141 ran the sweeps. T-3141 lists only the fresh case's 24 visibly-off CUMIPMT cells (C1, C2,
  C6, D1, ..., T9) and its own near-1E15 cells. T-4059 names type1 C7:C9 and G7:G9 and fresh I10 and K10. This task
  shares none of those cells (split note, T-3229: T-3141 and T-4059 own the cells they list, this task the rest).
- Distance from Excel:
  - fresh CUMPRINC (58): 42 at 1 ulp, 14 at 2, 2 at 3-4;
  - fresh CUMIPMT (32): 1 at 1 ulp, 9 at 2, 9 at 3-4, 13 at 5-16;
  - type1 CUMPRINC (40): 34 at 1 ulp, 4 at 2, 2 at 3-4.
  The CUMIPMT cells past 4 ulp follow from CUMIPMT = `PMT·m - CUMPRINC` (T-3118), which turns the principal's ulps
  into many of its own (financial.md "CUMIPMT and CUMPRINC over many periods").
- Examples (Excel vs ours):
  - fresh A1 `CUMPRINC(0.02,1E5,987.65,1E5-0,1E5,0)`: -19.365686274509805 vs -19.3656862745098;
  - fresh K4 `CUMIPMT(0.15,77777,1,77777-6,77777,0)`: -0.42593703992309273 vs -0.4259370399230926.
- Root cause: the same open question as T-3141, the operation order of `underflowed_principal` (balance drop
  `-(pv·(v^(nper-end)·(1-v^m)))` on Excel's EXP, Kahan's expm1 and `growth_log`) in
  `crates/rustsheet-engine/src/functions/financial/cumulative.rs`. T-3141 is the visible-parity owner and goes first.
  This task scores these cells under whatever form T-3141 lands, so it depends on T-3141. Type1 tests a fixed
  hypothesis: is type 1 the type 0 principal divided by 1 + rate (fin-cumprinc-near-1e15 F against A)?
- Fit plan (opus: numeric fitting):
  - The cached evidence is 200 fresh cells and 72 type1 cells, one excel-graph fill each.
  - The sweeps were drawn after the freeze of the core T-3141's draft (PR #3105) tried,
    `C = pv*((1-v^m)*v^(nper-end+1))` (the fresh case's description). Any form chosen after that, by T-3141 or here,
    needs a fresh sweep drawn after its own freeze (R4.2).
  - The sweep can be drawn inside this task, the way T-4412 does it: the fresh case's description gives the grid
    (five rates, each with its own nper and pv, and the last-m and inner-m rows).
  - No separate research sweep task is needed.
- Reproducibility floor (R4.4): most differences are 1-2 ulp, inside the 4-ulp web/desktop floor
  (docs/research/oracle-calibration.md). Refill both cases before fitting.
- Read docs/research/excel-kernel-playbook.md first. Decision 0056 applies: after two attempts that do not raise the
  bit-exact count, time-box.
- Cells:
  - probes/fin-cumprinc-underflow-fresh.case.json (90): Sheet1!A1, A2, A5, A6, B1, B2, B4, B5, B6, B8, C5, D4, D5, E1, E2, E3, E4, E6, E8, E9, F1, F2, F3, F4, F8, F9, F10, I2, I4, I5, I7, J1, J3, J4, J5, J6, K2, K4, K5, K7, L1, L3, L4, L5, L6, M1, M2, M3, M4, M6, M7, M8, M9, M10, N1, N2, N3, N4, N5, N6, N7, N8, N9, O1, O2, O3, O4, O6, O8, O9, O10, P1, P2, P3, P4, P5, P6, P8, P9, Q2, Q4, Q10, R2, R4, R8, R9, S4, S10, T4, T8
  - probes/fin-cumprinc-underflow-type1.case.json (40): Sheet1!A2, A3, A4, A5, A9, B1, B2, B4, B7, B9, C2, C3, C6, D2, D4, D6, D7, D8, D9, E2, E3, E4, E6, E8, E9, F1, F2, F4, F6, F7, F9, G1, G2, G4, G5, G6, H3, H4, H7, H8

## Acceptance criteria
- [ ] After T-3141, a refill of both cases (excel-graph) reports how many listed cells changed bits. Cells that vary
      between fills stop being targets (R4.4, decision 0112) and are listed in the Outcome.
- [ ] One of these holds:
      - an underflowed-principal form makes the listed CUMPRINC cells bit-exact, and the CUMIPMT cells follow through
        `PMT·m - CUMPRINC`;
      - or the Outcome lists every cell still off, with Excel, ours and the 60-digit value, and ranks the forms tried
        by bit-exact count.
- [ ] Any kept form or cut chosen by scoring passes every R4 guard of reviewer-excel-fidelity check 7 (decision 0188),
      each shown in the Outcome:
      - hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the
        candidate and for the correctly rounded reference, from the fit lab's T-3360 split, with the margin fixed
        before scoring;
      - a fresh sweep, drawn and oracle-filled after the freeze, and its result;
      - the Gate A envelope count on that sweep (T-4059's CUMPRINC and CUMIPMT tables once they exist), with every
        point outside the envelope listed;
      - a `FITTED(T-4524, n_cells, range)` tag, applied only inside its range;
      - no step that fits differences below the reproducibility floor;
      - no keep-matched branch.
- [ ] These known-failure lines disappear from eval/baseline/probes.txt with no new ones: the listed cells' `F` lines
      under probes/fin-cumprinc-underflow-fresh and probes/fin-cumprinc-underflow-type1. Every cell the Outcome lists
      as still off is excepted. No other eval/baseline/*.txt file gains a line, in particular probes.txt's other
      fin-cumprinc-* and fin-cumipmt-* cases and ironcalc-calc's CUMPRINC_CUMIPMT.xlsx.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells T-3141 lists, and its near-1E15 cells.
- Type1 C7:C9 and G7:G9 and fresh I10 and K10 (T-4059).
- Ranges where `v^nper` does not underflow (T-3216, T-4405, T-4406, T-4412).
