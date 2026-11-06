---
id: T-3337
title: "CHIINV and CHISQ.INV.RT at one degree of freedom land where Excel's search on its double cdf lands (CHIINV(1E-9,1) = 37.324892889999234, ours 37.324893106518715)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3336]
campaign: visible
---

## Goal
CHIINV and CHISQ.INV.RT at one degree of freedom (df 1 to 1.99, truncated) give Excel's value in the right tail,
where Excel's search on its own double-valued lower cdf of 1 - p lands a cdf step away from the crossing, so the
visibly-off pair below is within 15 digits of Excel (or the time-box outcome lists the forms tried, decision 0056).

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main, excel-graph; `scripts/exit_owners.py --all-off`: no open task
  named these; T-2561, which cited them as a lead, is done, and T-3311, T-3146 and T-3232 own other shapes and p):
  - probes-numeric/m9-legacy-chiinv Calc!A1 `=CHIINV(1e-09,1)` and Calc!A2 `=CHISQ.INV.RT(1e-09,1)`: Excel
    37.324892889999234, ours 37.324893106518715 (`NumericMismatch`, visibly off in the 8th digit).
- What is known (docs/excel-semantics/statistics.md "CHISQ.INV.RT and CHIINV quirks"): at df 1 the right-tail quantile
  is the lower quantile of 1 - p, `#NUM!` from p = 2^-54 down. Ours does exactly that (`chisq_inv_of` in
  crates/rustsheet-engine/src/functions/statistical/chisq.rs, the `is_one_degree` branch, `gamma_quantile(.., 1.0 - p,
  Tail::Lower)`) and returns the complement crossing. The true quantile is 37.32489305136234 (T-2561), so Excel is the
  inaccurate side: about 2.2E-7 below the crossing, one step of the double cdf's resolution near 1 (an ulp of 1 - p is
  about 1.1E-16, the density about 5E-10). At p = 1E-10 Excel is 2 ulp from ours (m9-chisq-inv-rt A1, 41.82145620298276
  against 41.82145620298279) and at 1E-4 1 ulp (m9-legacy-chiinv A12), so the step is not taken everywhere.
- Needs an oracle sweep: three points cannot say at which p Excel takes the step or what its search stops on. The fit
  waits for T-3336 (CHIINV and CHISQ.INV.RT at df 1 for p from 1E-4 to 2^-54, CHISQ.INV at 1 - p, and CHISQ.DIST at
  Excel's landing points). Read docs/research/excel-kernel-playbook.md first.
- Leads: Excel may stop when its cdf at the iterate first rounds to 1 - p (the lowest x of the flat band of the double
  cdf, as T-3146 suspects at p = 0.999 for other shapes), or bisect to a bracket width in x. Excel's lower cdf near 1 is
  1 - Q with Q from the complement (ERFC at df 1), so the band edges follow ERFC's last digits: check
  `CHISQ.DIST(x,1,TRUE)` bit-exactness at the landing points first (T-3336's CHISQ.DIST cells).
- The 1-to-4-ulp df-1 cells (m9-chisq-inv-rt A1-A7, m9-legacy-chiinv A11-A62, t1049-chisqinvrt, t3148-gamma-quantile-flat-band
  A64-A112) are owned by T-3232 and T-3146; if the landing rule moves them, record which (they may pass with it).
- A 100-digit mpmath check (draft PR #2503, 2026-10-31): Excel's `CHIINV(1e-09,1)` (A1) is, to the bit, the exact
  quantile of the cdf value one double below q = fl(1-1E-9); ours is the exact quantile of q itself.
  `NORM.S.INV((1+q)/2)^2` matches A1 but is 114 to 134,935 ulp off at p = 0.001, 0.995, 0.999 and 0.99999. That rules
  out a df-1 normal closed form and points at the landing rule above.

Cells:
- probes-numeric/m9-legacy-chiinv.case.json (2): Calc!A1, A2
- probes-numeric/t3336-chisq-df1-right-tail.case.json (291, T-3336)

## Acceptance criteria
- [ ] Visible parity: the 2 cells above are within 15 digits of Excel (`make eval-show
      CASE=probes-numeric/m9-legacy-chiinv.case.json`; probes-numeric is ungated, so they have no baseline line), no new
      known-failure line appears in any eval/baseline file (`make eval-baseline`), and every T-3336 cell is within 15
      digits of Excel or named in a follow-up task.
- [ ] Or, time-boxed (decision 0056, two attempts): the Outcome and statistics.md list the candidates with their
      bit-exact and visible counts on the T-3336 cells.
- [ ] No df-1 cell that passes today regresses (t1543-inverse-cdf-tiny-p A37-A40, m9-legacy-chiinv A71, A72).
- [ ] The search stays bounded (a fixed number of cdf evaluations, directive 5).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every
visibly-off cell it owns is within half a unit of the 15th significant digit of Excel's value. Excel is the side off
the true quantile here (ours is the correctly rounded crossing), so this is the `visible` stage: reproduce Excel's
landing, never a more accurate kernel. Bit-level misses left then go to a `defer: phase-1b` follow-up.

## Out of scope
- df 2 and above (T-3311 deep lower tail, T-3146 p = 0.999, T-3310 far right tail, T-3232 last digits).
- The `#NUM!` edge between 2^-54 and 1E-16 beyond recording what T-3336 finds.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 33 probes-numeric cells that no open task listed: 25 first wrong at the 11th significant digit or earlier, 8 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0267 rules the math (decision 0188 R3) on the CHISQ.INV.RT and CHIINV df 1 cells where Excel is more than G = 1e-4 off the 50-digit math. The decision 0267 analysis (mpmath at 60 digits) found 6 listed cells above G: CHISQ.INV.RT and CHIINV at p 1.5E-16, 3E-16 and 5E-16. For example, at `CHISQ.INV.RT(1.5E-16,1)` Excel is 67.39648382445009, the math 68.169874129795366 and ours 68.7632522116684, so Excel is 1.1E-2 off and ours 8.7E-3. The engine is just as wrong there, so those cells need a more accurate df 1 inverse plus R3 marks in eval/accuracy/math-over-excel.tsv. Each mark names one point whose committed 50-digit value (a math.json sidecar) shows Excel more than G off. The analysis numbers above are not evidence until committed, and the marks are not a band. Below G, ours is also up to 1E-5 off the math at p 1E-13 to 2E-13, so the search itself needs work. On the other cells Excel's landing point stays the target.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3336-chisq-df1-right-tail.case.json (33): Calc!A19-A20, A37-A38, A43-A44, A55-A56, A67-A68, A73-A74, A85-A86, A91-A92, A97-A98, A124-A125, A163-A164, A169-A170, A196-A197, A220-A221, A226-A227, A229-A230, A290

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).
