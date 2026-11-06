---
id: T-3386
title: "T.INV.2T and TINV at a tiny p: Excel is over 1e-4 off the math from p 1.7E-110 down (R3 case, owner ruling in decision 0267)"
milestone: M9
lane: functions-stat
status: done
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
- [x] A `MATH-OVER-EXCEL` entry in statistics.md and the R3 marks in eval/accuracy/math-over-excel.tsv (SPEC-0029)
      for every listed cell where Excel is more than G off the math. A cell whose Excel value is within G keeps Excel
      as its target, and this task names it, or blocks it under decision 0056.
- [x] df 1 near p = 1 (`T.INV.2T(0.999999999999999,1)`) is outside the ruling and stays Excel's (T-2858).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The T.INV points of R3 case 1 (the Gate A tables, T-3358) stay with T-3358. This task owns only the t1531, t1543,
  t3349 and m9 cells listed under "Decision 0267 cells".

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 58 probes-numeric cells that no open task listed: 3 with the wrong type or error, 54 first wrong at the 11th significant digit or earlier, 1 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t1531-t-tiny-p.case.json (10): Calc!A73-A74, A82-A84, A97-A98, A106-A108
- probes-numeric/t1543-inverse-cdf-tiny-p.case.json (6): Calc!A43, A79, A86, A89, A91-A92
- probes-numeric/t3349-t-inv-2t-sweep.case.json (42): Calc!A211, A214-A217, A219, A224-A225, A230, A232, A237-A240, A242-A243, A246, A252, A257, A262-A264, A266, A270, A272-A274, A277, A279-A281, A283-A285, A287-A289, A291, A293-A294, A296, A300
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A160 (from T-0967, 2026-11-05: `TINV(1E-200,1E+10)`, Excel 58.19256428927105,
  ours 30.228508783079672, about the normal quantile; Tier A, the tiny-p case of decision 0267 item 3)

- [ ] The 3 type cells listed (t1543 A89, A91 and A92: the engine returns `#NUM!` where Excel returns a number for `BETA.INV(1-1E-16,1000,1)`, `BETA.INV(1E-300,0.5,100)` and `BETA.INV(1E-200,0.5,100)`) never pass through an R3 mark while they are an error against a number (decision 0188 R3). The engine returns a number there. That is Excel's number, or the math where a mark applies.
- [x] Each R3 mark names one point whose committed 50-digit value (a math.json sidecar, or a Gate A table's math
      column) shows Excel more than G = 1E-4 off. Marks go point by point: "tiny p" is not a band, and the
      decision 0267 analysis numbers do not count as evidence until committed.
- [ ] Every cell listed above does one of three things: it is an R3-marked cell where the engine returns the math within ε (R1(c) `R3Pass`); it matches Excel within half a unit of the 15th significant digit or has Excel's type and error; or it is left after decision 0056's time-box and accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" either passes as an R3 cell (decision 0188 R3, ruled by decision 0267), or is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text.

## Outcome
Decision 0311 records the marks (a protected path). 52 cells got their own line in eval/accuracy/math-over-excel.tsv
under statistics.md "Tiny-p quantiles: Excel is grossly wrong, RustSheet returns the math", whose Rule and new T-3386
Region name them. Each was checked against its committed math sidecar value: Excel 0.67 to 2.05 relative off (G 1e-4),
the engine within ε (worst 5.3E-10, t3349 A266). No engine change was needed.
- t3349-t-inv-2t-sweep: all 42 listed T.INV.2T cells (p 1.38E-300 to 1.73E-110, df 2490 to 9987930597).
- t1531-t-tiny-p: A73, A74, A97, A98 (T.INV), A82, A106 (T.INV.2T), A83, A107 (TINV).
- t1543-inverse-cdf-tiny-p: A86 (`BETA.INV(1E-200,1000,1)`, math (1E-200)^(1/1000)).
- gen-m9/7980ffb6095a340d A160 (`TINV(1E-200,1E+10)`): gen-m9 had no sidecar, so this PR commits that case's sidecar
  as `rustsheet-accuracy math --suite gen-m9 --function TINV` writes it (only that case's; the run wrote 103).

Not marked:
- t1531 A84 and A108 (CONFIDENCE.T): no committed math, since CONFIDENCE.T has no Gate A definition (T-4102). Follow-up
  T-4535.
- t1543 A89, A91, A92: engine `#NUM!` against Excel numbers, so no mark applies (they take none). The engine does not
  return a number yet: that criterion and the last one stay open and move to T-4536.
- t1543 A43 and A79: Excel within G, so Excel stays the target (the engine is on the math, Excel 6.0E-12 and 1.1E-9
  off). Follow-up T-4536; not attempted here, so decision 0056's time-box has not started for them.
- The Gate A points T.INV.2T/s1 A98, A99, A103 and TINV/s1 A102 stay unmarked (decision 0301). Gate A's four known
  failures this task was listed for (T.INV.2T/s1 and TINV/s1 A68, A78: Excel `#NUM!` at p = 2.2E-308, an error
  mismatch) need a probe of where Excel's cut is, so their known-failures.tsv lines now name follow-up T-4541.

Eval delta: no cell changes verdict and no case's `passed` changes (t1531 75/105, t1543 64/102, t3349 134/300, gen-m9
7980ffb6095a340d 393/400, before and after); eval/baseline is unchanged. R1(c): the 52 cells move from counted to
`r3_pass` (probes-numeric counts 1636 → 1585, r3_pass 66 → 117), and the new gen-m9 sidecar moves
gen-m9/7980ffb6095a340d A84 to Phase 1b. Gate A (`rustsheet-eval accuracy`): 0 failing, 285 known failures, unchanged.
