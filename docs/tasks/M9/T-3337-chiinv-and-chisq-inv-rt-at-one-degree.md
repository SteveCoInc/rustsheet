---
id: T-3337
title: "CHIINV and CHISQ.INV.RT at one degree of freedom land where Excel's search on its double cdf lands (CHIINV(1E-9,1) = 37.324892889999234, ours 37.324893106518715)"
milestone: M9
lane: functions-stat
status: blocked
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
- probes-numeric/t3858-chisq-df1-num-cutoff.case.json (264, T-3858): the df 1 `#NUM!` cutoff, pinned between
  8.3253E-17 (`#NUM!`) and 8.3254E-17 (a value) and implemented in T-3858 (`ONE_DEGREE_RIGHT_TAIL_P_MIN`), which
  fixes T-3336's A250-A252; it is in INV.RT's own p check, as `CHISQ.INV(1-p,1)` is a value at every probed p.

## Acceptance criteria
- [ ] Visible parity: the 2 cells above are within 15 digits of Excel (`make eval-show
      CASE=probes-numeric/m9-legacy-chiinv.case.json`; probes-numeric is ungated, so they have no baseline line), no new
      known-failure line appears in any eval/baseline file (`make eval-baseline`), and every T-3336 cell is within 15
      digits of Excel or named in a follow-up task.
- [x] Or, time-boxed (decision 0056, two attempts): the Outcome and statistics.md list the candidates with their
      bit-exact and visible counts on the T-3336 cells.
- [x] No df-1 cell that passes today regresses (t1543-inverse-cdf-tiny-p A37-A40, m9-legacy-chiinv A71, A72).
- [x] The search stays bounded (a fixed number of cdf evaluations, directive 5).
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
- probes-numeric/t3337-chisq-df1-r3-fresh.case.json (T-3337's fresh sweep): Calc!A43:A44, A51:A52 are visibly off
  with Excel the target (Excel within G of the math); A1:A30 are R3-marked (decision 0313).

## Outcome
- **R3 cells (decision 0267 item 3, decision 0313).** At df 1 CHISQ.INV.RT and CHIINV return the upper-tail quantile
  of p (`gamma_quantile(.., p, Tail::Upper)`). They do so where the quantile of the double 1 - p, Excel's own route,
  is more than G = 1e-4 off it (chisq.rs `one_degree_right_tail_quantile`). Otherwise they keep the quantile of
  1 - p as before. The rule is skipped from p 1E-12 up, where the rounding cannot reach G. It costs at most two
  bounded searches.
- **Marks.** There are 45 lines in eval/accuracy/math-over-excel.tsv, each checked against its committed math
  sidecar (Excel 3.9E-4 to 1.1E-2 off, engine at most 2.5E-16 off):
  - t3336 CHISQ.INV.RT A202, A217, A220, A223, A226, A229, A232, A253 and CHIINV A203, A218, A221, A224, A227,
    A230, A233;
  - t3337-chisq-df1-r3-fresh A1:A30.

  0267's analysis named only 6 of these, the ones visibly off then. On every evidenced point Excel is more than G
  off, and the ruling covers "each evidenced point", so the others are marked too.
- **CHIINV math values.** CHIINV gets a Gate A definition, CHISQ.INV.RT's legacy name with no reference table
  (tools accuracy/defs/gamma.py). `rustsheet-accuracy math --function CHIINV` rewrote the sidecars of the probes and
  probes-numeric cases that call it (7 existing sidecars changed). The diff adds only CHIINV entries.
- **R4 guards (structural rule, no fitted value; G is decision 0188's constant).**
  - Hold-out: none was split, because no candidate was scored to pick the rule. It applies the ruling's own test to
    the engine's two quantiles.
  - Fresh sweep: probes-numeric/t3337-chisq-df1-r3-fresh, 52 cells at p 1.2E-16 to 1.2E-12 not in t3336, filled
    from excel-graph after the rule was committed (aede4ec4f). The rule fires at A1:A30, and Excel is more than G
    off at exactly those cells (3.9E-4 to 7.8E-3). It keeps Excel's route at A31:A52, where Excel is within
    2.8E-5. That is 52 of 52 in agreement.
    The sweep covers p 1.2E-16 to 1.2E-12, not the full domain (R4.2), because the rule cannot act from p
    1E-12 up: there the engine returns the same computation as before.
  - Over t3336 and the fresh sweep (225 numeric cells with a math value), the rule fires at exactly the 45 cells
    where Excel is more than G off. It also fires at t3336 A250:A252, where Excel is `#NUM!` (T-3858's cut).
  - Gate A: `rustsheet-eval accuracy` reports 0 failing, 0 bad marks and 0 table defects.
- **Eval delta (probes-numeric is ungated, so no baseline change).**

  | case | passed before | passed after |
  |---|---|---|
  | t3336-chisq-df1-right-tail | 79/291 | 76/291 |
  | t3337-chisq-df1-r3-fresh (new) | - | 4/52 |
  | m9-legacy-chiinv | 49/89 | 49/89 |
  | m9-chisq-inv-rt | 17/49 | 17/49 |
  | t1543-inverse-cdf-tiny-p | 64/102 | 64/102 |

  Per-cell changes, all in t3336 and all R3 cells:
  - A217, A218 and A253 passed against Excel and now fail against it.
  - A202, A203, A223, A224, A232 and A233 were within 15 digits and now fail against Excel.
  - A220, A221, A226, A227, A229 and A230 failed and still fail, with the engine now on the math instead of
    8.7E-3 to 3.1E-3 off it.
  - A250:A252 still fail, now on the math rather than 68.7632522116684, with Excel `#NUM!` (T-3858).

  Every one of these is `r3_pass` under R1(c). t1543 A37:A40 and m9-legacy-chiinv A71, A72 still pass.
- **Visible parity (Excel the target), time-boxed.** These cells are unchanged: t3336 A19:A20, A37:A38, A43:A44,
  A55:A56, A67:A68, A73:A74, A85:A86, A91:A92, A97:A98, A124:A125, A163:A164, A169:A170, A196:A197, A290 and
  m9-legacy-chiinv A1, A2. Candidates were scored on the 110 CHISQ.INV.RT and CHIINV cells of t3336 with p of
  1E-12 or more, as bit-exact / within 15 digits:
  - H0, the quantile of fl(1-p), which is the engine: 16 / 87.
  - H1, the quantile of the double below fl(1-p): 5 / 13.
  - H2, the quantile of 2*fl((1+q)/2)-1: 13 / 74. It fixes 11 cells and loses 24.
  - "A last-bit error of Excel's own cdf" is refuted by t3336 A256.

  The details are in statistics.md, "CHISQ.INV.RT and CHIINV quirks". The scratch scripts were mpmath erfinv at 60
  digits over the oracle and math sidecars. They are not committed.
- **Merged with T-3858** (main at 0db7e8ac9, #3535): its `#NUM!` cut runs first, then this task's rule. Above the cut
  Excel's 68.7632522116684 is more than G off, so T-3858's unit-test rows A4 to A6 now expect the math, and T-4555's
  marks are folded in here: 88 cells of t3858-chisq-df1-num-cutoff (54) and -fresh (34), with CHIINV math added to
  both sidecars. Those cases pass 210/264 and 46/80 against Excel, all 88 failures R3 cells; `rustsheet-eval
  accuracy` 0 failing.

## Blocked because: time-boxed (decision 0056)
Two landing rules (H1, H2 above), plus the refuted cdf-last-bit idea, do not move the Excel-target cells (H0, the
engine, stays best). Decision 0267 item 4 applies: the cells this task still lists are accepted, and no follow-up is
filed for them. The R3 part of the task is done (decision 0313). Reopen only on a new hypothesis recorded here
(decision 0056). Excel's landing sits within a few ulp of the exact quantile of fl(1-p), or about one band from it
in either direction, or inside a band, so its search does not stop on a fixed band rule.
