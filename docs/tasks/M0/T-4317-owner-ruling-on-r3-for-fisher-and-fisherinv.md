---
id: T-4317
title: "Owner ruling on R3 for FISHER and FISHERINV near 0, where Excel's LN and EXP quotients cancel (T-3785 follow-up)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's FISHER(x) and FISHERINV(y) near 0, where 0.5*LN((1+x)/(1-x)) and
(EXP(2y)-1)/(EXP(2y)+1) cancel in doubles, is a decision 0188 R3 (`MATH-OVER-EXCEL`) case. Until the owner rules, the
engine keeps matching Excel.


## Context
- statistics.md "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)" and
  "FISHERINV is (EXP(2y)-1)/(EXP(2y)+1), with its cancellation near 0": both carry an absolute error near 1E-16, so
  about 1E-16/|x| relative, which passes R3's G = 1e-4 below about |x| = 1E-12.
- Evidence, probes/t2437-fit-separators (excel-graph; math from the T-3785 sidecar, Excel against atanh/tanh):
  - past 1e-4: FISHER A2 (`FISHER(1.39727256563331E-15)`, 4.7E-2), A4 and A7 (Excel 0, math 1.03E-16 and 1.07E-16),
    A5 (2.0E-2), A8 (8.4E-3), A10 (6.4E-3), A11 (6.7E-3); FISHERINV A21 to A32 and A34 to A36 (|y| from 1E-16 to
    8E-15, 2.2E-3 to 0.20).
  - inside R3: FISHER A1, A3, A6, A9, A12, A20 (`FISHER(-1E-12)`, 3.3E-5) and A19 (`FISHER(1E-08)`, 5.0E-9);
    FISHERINV A33 (`FISHERINV(1E-10)`, 8.3E-8).
- The engine matches Excel at these cells (directive 1; fit lab `fisher`, `fisherinv`). T-3785's Gate A tables leave the
  region out: FISHER and FISHERINV points with 0 < |x| < 1E-6 are redrawn. So the gate does not depend on this ruling.
- The owner's options:
  - (a) keep matching Excel, so this is no R3 case;
  - (b) an R3 case over a bounded region of x, with a decision record.

## Acceptance criteria
- [x] A decision record carrying the owner's ruling, or the owner's "keep matching Excel" written into the
      statistics.md T-2437 entry.
- [x] If (b): a follow-up implementation task with its region, and the region's evidence committed first
      (decision 0188 R3).

## Out of scope
- Changing the engine before the ruling.
- GAUSS's cancellation (T-4277) and ERF's two-argument one (T-3794).

## Outcome
- Owner ruling 2026-11-06, decision 0300: keep matching Excel. This is not an R3 case: no `MATH-OVER-EXCEL`
  entry, no R3 mark, no engine change (the engine already matches Excel here).
- Eval delta: none.
