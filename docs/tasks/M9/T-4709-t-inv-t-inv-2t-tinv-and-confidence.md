---
id: T-4709
title: "T.INV, T.INV.2T, TINV and CONFIDENCE.T last ulp at p from 2.2E-308 to 1E-300: the 24 t4541 cells no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1542]
defer: phase-1b
---

## Goal
The 24 cells of probes/t4541-t-inv-2t-tiny-p that are one ulp from Excel return Excel's bits, or the Outcome records
why a cell stays. The cells are Student-t quantiles at p from 2.2E-308 to 1E-300 and df 2 to 31, with values from
9E10 to 1E307.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/functions-lookup/T-4570-xlookup-ref-element, based on origin/main
  5aed684f1). I checked every failing cell of report.json against open tasks with `scripts/exit_owners.py`'s `owned`
  rule. T-4541 (done) committed this probe and recorded its 24 last-ulp `NumericUlp` misses. No open task names them:
  T-4691 covers only T-4541's docs and Gate A follow-ups, and T-1542 lists tiny-p cells at p of 1E-300 and above
  from other cases. Every cell is within 15 digits (`ulp1`), so this task is `defer: phase-1b` (decision 0114).
- Cells (probes/t4541-t-inv-2t-tiny-p.case.json, Calc!):
  - T.INV.2T: C5, C9 (df 9), D5 (df 30);
  - TINV: F5 (df 30);
  - CONFIDENCE.T: G7 (n 2), H6, H7, H9, H11, H12 (n 3), I5, I6, I7 (n 31);
  - T.INV: K6, K7, K10, K11 (df 2), L1, L2, L3, L4, L5, L6, L12 (df 9).
- Examples (Excel vs ours, `make eval-show CASE=probes/t4541-t-inv-2t-tiny-p.case.json`; math from the eval sidecar):
  - K6 `T.INV(4.4502E-308,2)`: Excel -3.3519322922951695E+153, ours -3.351932292295169E+153. Both are 2.0E-14 off the
    math.
  - H6 `CONFIDENCE.T(4.4502E-308,1,3)`: Excel 2.736841256160322E+153, ours 2.7368412561603215E+153. Both are 5.0E-14
    off the math.
  - C5 `T.INV.2T(4.45014771701441E-308,9)`: Excel 3.648002741446659E+34, ours 3.6480027414466594E+34. Excel is 8.8E-16
    off the math and ours 1.0E-15.
- Both values are equally far from the math, by much more than an ulp at df 2 and 3. So ours already follows Excel's
  formulation, and only the last rounding differs. Suspected cause, not yet traced: q < `T_TAIL_P` takes
  `t_dist.rs` `t_tail_closed_form`, which returns sqrt(df)·exp(-ln_x/2)·sqrt(1 - x) with ln_x near -706 at df 2. An
  ulp of ln_x there is about 4E-14 relative in t, which matches the shared error. That points to Excel taking a log
  form of the same size and differing in operation order. Candidates to score: the order of the ln terms, pow
  against exp of a scaled log, the x87 register rounding (numbers.md), and CONFIDENCE.T's scaling by s/sqrt(n).
  This is a structural model, unless a constant gets chosen by scoring.
- Depends on T-1542, which works the same tiny-p path at p of 1E-300 and above. Re-measure these cells after it lands:
  its operation order may already fix them. Otherwise they are its natural hold-out below 1E-300.
- statistics.md: "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308 (T-3358)", which has the
  T-4541 two-tailed flush rule.

## Acceptance criteria
- [ ] Each cell above is bit-exact, or the Outcome lists it with its Excel, ours and 50-digit values and the reason it
      stays. These known-failure lines disappear from eval/baseline/probes.txt with no new ones.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), for any operation order or constant chosen by
      scoring:
  - hold-out counts (exact / 4 ulp / 15 digits) on a hold-out split fixed before scoring, for the chosen candidate
    and the correctly rounded one;
  - a fresh sweep, drawn after the formulation is frozen, is scored (an excel-probe case of T.INV, T.INV.2T and
    CONFIDENCE.T at p from 2.2E-308 to 1E-290 and df 2 to 40, in probes);
  - the Gate A envelope result is in the Outcome (within 15 digits of Excel, or within 1E-9 of the 50-digit value),
    listing every point outside, and `rustsheet-eval accuracy` reports 0 failing;
  - the code carries a `FITTED` tag with the fitted range (none for a structural model, decision 0188 R4 (c));
  - the reproducibility floor is met;
  - there is no branch that keeps already-matched cells by special case.
- [ ] No gated or probes-numeric cell goes from within 15 digits to visibly off. In particular, T-1542's and T-3061's
      cells do not move away from Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The two-tailed flush rule and its errors (T-4541, done), and its docs (T-4691).
- Tiny-p cells where Excel is more than 1E-4 off the math (decision 0267 item 3, the R3 marks of T-3386).
