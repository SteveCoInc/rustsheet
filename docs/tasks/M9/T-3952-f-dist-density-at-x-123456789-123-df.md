---
id: T-3952
title: "F.DIST density at x = 123456789.123, df (8, 8): Excel 2.2E-15 and ours 6.6E-15 off the math (gen-m9 2801c1c11c21e134 A219, the Gate B face of deferred T-3173)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2586]
campaign: visible
---

## Goal
The gen-m9 cell below is within half a unit of the 15th significant digit of Excel's value, by porting the power-term
step of Excel's F density chain that T-2586 identifies, with no other F.DIST density cell moved off.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Gap triage 2026-11-03 (`make eval` on origin/main c2062e5c8, excel-graph). `scripts/exit_owners.py --all-off` lists
  this cell as visibly off and owned only by a `defer: phase-1b` task: T-2221's "Cells added from T-3198 (2026-11-03)",
  which says none of those cells is visibly off. The report disagrees for this one, and gen-m9 is a Gate B suite
  (decision 0188 R1(b)), so a deferred owner leaves the `owners` line of `make phase1-exit` unmet. This task takes it
  out of T-2221's list (T-2221 names this task).
- gen-m9/2801c1c11c21e134 Calc!A219 `=F.DIST(Data!B9,8,8,REDUCE("ü",Data!A6:D11,LAMBDA(w,x,ISBLANK(x))))` with
  Data!B9 = 123456789.123 (the REDUCE gives FALSE, so this is the density): Excel 4.881498042869914E-39, ours
  4.881498042869936E-39 (`make eval-show CASE=gen-m9/2801c1c11c21e134.case.json`, NumericClose).
- 50-digit reference (mpmath 1.3 in the tools venv, dps 50, the closed-form density at the double x):
  4.8814980428699033522E-39. Excel is +2.2E-15 relative from it, ours +6.6E-15. Half a unit of Excel's 15th digit is
  1.0E-15 relative here, so even a correctly rounded density stays visibly off: the target is Excel's own arithmetic
  (`visible`, not `accuracy`).
- Region: side 1 at r = fl(d1·x) far above d2, power term P = c·LN x with c = d1/2 = 4 and LN x = 18.63, so |P| = 74.5.
  T-3173 (deferred) measured M-final's parity at 36% for |P| in [32, 256): the power term is where the engine's model
  (`crates/rustsheet-engine/src/functions/statistical/f_density.rs` `excel_f_density`, T-3034) still misses Excel by
  1.35-2 times one rounding of P. T-2586 (research, in progress) is cracking that step (its round 5 "Next": Excel's
  exponential step, exp2 split, x87 F2XM1/FSCALE, or value-space powers with per-factor roundings). No form is fitted
  here blind (decision 0056); this task ports T-2586's result and checks it on this cell.
- Rule: docs/excel-semantics/statistics.md "F density is not the beta density, R's df or a GAMMALN formula (T-2513)".

Cells this task owns:
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A219

## Visible-parity stage
`campaign: visible` (decisions 0083, 0114). Done when Calc!A219 is within half a unit of the 15th significant digit of
Excel's value and no F.DIST density cell in any suite goes from within 15 digits to visibly off. Bit-level criteria
still unmet then go back to T-2221 (`defer: phase-1b`).

## Acceptance criteria
- [ ] gen-m9/2801c1c11c21e134 Calc!A219 leaves eval/baseline/gen-m9.txt or is within 15 digits of Excel
      (`make eval-show` output in the Outcome).
- [ ] The R4 guards of reviewer-excel-fidelity check 7 (decision 0188 R4) for the ported power-term step: hold-out
      counts (bit-exact, 4 ulp, 15 digits) beside the correctly rounded candidate from T-2586's split, a fresh
      log-uniform sweep filled after the model is frozen with its Gate A envelope result (every point outside listed),
      a `FITTED(T-XXXX, n_cells, range)` tag and range on any fitted piece (none for a structural model of kind (c)),
      nothing fitted below the reproducibility floor (4 ulp web against desktop, decision 0112), and no keep-matched
      branch.
- [ ] The F.DIST density counts within 15 digits on probes-numeric t2220-f-dist-sweep-pdf and the t2586-r5-* cases do
      not fall; no new line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 18,073 probes-numeric density cells of T-3173 and T-3174's |P| < 4 constant (Phase 1b), and the F cdf and tails.
