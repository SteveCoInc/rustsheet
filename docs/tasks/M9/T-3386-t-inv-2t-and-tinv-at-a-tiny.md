---
id: T-3386
title: "T.INV.2T and TINV at a tiny p: Excel is over 1e-4 off the math from p 1.7E-110 down (R3 case, owner ruling in decision 0267)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3358]
campaign: visible
---

## Goal
Apply the owner's ruling in decision 0267: at a tiny p, the Student-t, beta and chi-square quantiles follow the math
(decision 0188 R3) on every evidenced point where Excel is more than G = 1e-4 relative off the 50-digit math. That
covers T.INV.2T, TINV, T.INV and CONFIDENCE.T, plus BETA.INV in t1543. Each such point gets an R3 mark, and the engine
returns the math within ε there. Every other point keeps Excel as its target (directive 1).

## Context
- Gate A known failures this task owns (T-3358, eval/accuracy/known-failures.tsv): accuracy/T.INV.2T/s1 A68 and A78, and accuracy/TINV/s1 A68 and A78 (Excel #NUM! at p = 2.2E-308; error mismatches, which R3 never excuses). The R3 candidates of this task's region are T.INV.2T/s1 A98, A99 (p 4.4E-242) and A103 and TINV/s1 A102, which are not marked (decision 0213). Removing the lines
  once they pass is part of this task (a passing listed point fails the check as stale).
- T-3349's fresh sweep (probes-numeric/t3349-t-inv-2t-sweep, excel-graph 2026-10-31, mpmath 60 digits): 42 of 300
  points have Excel more than 1e-4 relative off the math, every one at p from 1.38E-300 to 1.73E-110 and df from
  2490 to 1.3E8 (A211-A300, extreme and log-uniform strata). Example `T.INV.2T(6.68863396408457E-199,2490)`: Excel
  100.83724584548176, math 33.047211122914894, engine 33.047211122914874. The engine returns the math there, so these
  cells are visibly off Excel but inside Gate A's envelope.
- R3's initial case 1 lists T.INV (left tail), F.INV, BETA.INV and CHISQ.INV.RT below p 1e-150, not T.INV.2T or
  TINV, and these points reach p 1.7E-110, above that bound. A new R3 case or a wider region needs a decision record
  with an owner ruling (decision 0188 R3); no agent adds one alone.
- docs/excel-semantics/statistics.md "Two-tailed Student-t quantile with p near 1 is accurate in Excel (T-3349)".

- Another point over 1e-4, at df 1 near p = 1 (gap triage 2026-10-31i): `T.INV.2T(0.999999999999999,1)` is
  1.6155696572206502E-15 in Excel against the exact 1.5695408241038844E-15 (2.9E-2 relative), from Excel's 1/TAN form
  (statistics.md "Two-tailed Student-t quantile with p near 1 is accurate in Excel (T-3349)"). T-2858 (branch
  agent/functions-stat/T-2858-t-inv-df1) takes the cell
  and matches Excel per directive 1; if the ruling here is the math, its region should say whether df 1 near p = 1
  is in it.

## Acceptance criteria
- [x] A decision record with the owner's ruling: decision 0267 rules the math. The region is the evidenced points
      where Excel is more than G off the math, checked per cell. The decision 0267 analysis (2026-11-04, mpmath at 60
      digits) found 50 of the 59 visibly-off tiny-p T.INV, T.INV.2T and TINV cells of t1531, t3349 and the m9 cases above G.
- [ ] A `MATH-OVER-EXCEL` entry in statistics.md and the R3 marks in eval/accuracy/math-over-excel.tsv (SPEC-0029)
      for every listed cell where Excel is more than G off the math. A cell whose Excel value is within G keeps Excel
      as its target, and this task names it, or blocks it under decision 0056.
- [ ] df 1 near p = 1 (`T.INV.2T(0.999999999999999,1)`) is outside the ruling and stays Excel's (T-2858).
- [ ] `make check` green; reviewers approve.

## Out of scope
- T.INV's own region (R3 case 1, T-3358).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 58 probes-numeric cells that no open task listed: 3 with the wrong type or error, 54 first wrong at the 11th significant digit or earlier, 1 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t1531-t-tiny-p.case.json (10): Calc!A73-A74, A82-A84, A97-A98, A106-A108
- probes-numeric/t1543-inverse-cdf-tiny-p.case.json (6): Calc!A43, A79, A86, A89, A91-A92
- probes-numeric/t3349-t-inv-2t-sweep.case.json (42): Calc!A211, A214-A217, A219, A224-A225, A230, A232, A237-A240, A242-A243, A246, A252, A257, A262-A264, A266, A270, A272-A274, A277, A279-A281, A283-A285, A287-A289, A291, A293-A294, A296, A300

- [ ] Every cell listed above does one of three things: it is an R3-marked cell where the engine returns the math within ε (R1(c) `R3Pass`); it matches Excel within half a unit of the 15th significant digit or has Excel's type and error; or it is left after decision 0056's time-box and accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" either passes as an R3 cell (decision 0188 R3, ruled by decision 0267), or is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text.
