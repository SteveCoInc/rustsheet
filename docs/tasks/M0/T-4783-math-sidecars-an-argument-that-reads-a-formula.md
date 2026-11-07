---
id: T-4783
title: "Math sidecars: an argument that reads a formula cell holding one defined call is a composite inner node (t3444 BINOM.DIST over BINOM.INV, 15 visibly-off cells)"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
A cell such as `=BINOM.DIST(A36,100000,0.001,TRUE)`, where A36 is `=BINOM.INV(100000,0.001,0.3)`, gets the same math
value as the inline composite `=BINOM.DIST(BINOM.INV(100000,0.001,0.3),100000,0.001,TRUE)` gets under decision 0297
option (b). The referenced formula's call becomes an inner node, rounded to its double. It is never read from the
engine's or Excel's value.

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main b9ae5216, every suite fetched). `make unowned-off ALL=1` lists 17
  visibly-off cells with no open owner, all in probes/t3444-binom-range-inv-large-n. 15 of them are column C
  (`BINOM.DIST(A<r>,n,0.001,TRUE)`) and column D (`BINOM.DIST(A<r>-1,n,0.001,TRUE)`) of sheet Inv, where A<r> is
  `=BINOM.INV(n,0.001,alpha)`. They have no math value, so decision 0188 R1(c) counts them, and they keep the M9 probe
  row from being met. Decision 0325 says such cells belong to the task that attaches their math. That is this task,
  and no `visible` or `accuracy` fit task may take them first. T-4782 attaches the other two (Range!B49, B53: "not
  converged").
- Why there is no value: tools/src/rustsheet_tools/accuracy/eval_math.py (module docstring, and the `formulas` cache
  near line 116) reads an argument that refers to a formula cell only when that formula is arithmetic over number
  literals (an exact-input cell). A cell holding a call is "unparsed". T-3924 and T-3993 (both done) left "arguments
  that read formula results (t3444's `BINOM.DIST(A1-1,...)` over BINOM.INV)" out of scope. No open task covers them.
- Math at the BINOM.INV integers that Excel returns (triage scratch run: a direct pmf sum at 40 digits in mpmath; the
  BINOM.INV cells themselves match Excel). Excel is 2.5E-13 to 1.3E-12 relative below the math and the engine is
  within about 1E-14, so both are within 1e-9. Once the math is attached, these cells become Phase 1b evidence, not
  R1(c) counts:

  | cell | k, n | Excel | engine | math |
  |---|---|---|---|---|
  | Inv!C36 | 95, 1E5 | 0.3311016442530485 | 0.33110164425313326 | 0.33110164425313579 |
  | Inv!D49 | 999, 1E6 | 0.4957884448324078 | 0.49578844483300816 | 0.49578844483300771 |
  | Inv!C54 | 983, 1E6 | 0.30215824528808366 | 0.30215824528837626 | 0.30215824528837574 |
  | Inv!D67 | 9999, 1E7 | 0.4986681958001577 | 0.49866819580081084 | 0.49866819580081081 |

  At the median rows, where BINOM.INV's alpha is 0.5 (C31, C49, C67), Excel is within 2E-15 of the math. The
  others, D31, D49 and D67 one count below among them, are off by up to 1.3E-12. This is the small-p regime of
  T-3961. No Inv!A or Inv!B cell fails, so the BINOM.INV integers agree with Excel.
- Cells this task names (probes/t3444-binom-range-inv-large-n). Their bit-level owner is T-4784, which is deferred:
  - probes/t3444-binom-range-inv-large-n.case.json (15): Inv!C36, C50, C54, C68, C72, D31, D36, D49, D50, D52, D54,
    D67, D68, D70, D72
- Design points to settle in this task, recorded in SPEC-0029 "Math references for eval cells", with a one-line
  amendment to decision 0297 if the reviewers ask for one:
  - The referenced cell's formula must itself be one call (or a composite) over literals and exact-input cells. Chains
    are followed to `MAX_NESTING`, and a cycle is unparsed.
  - The math of the inner node must round to the referenced cell's own double for the read to be faithful. An
    integer-valued inner call (BINOM.INV, CRITBINOM) is exact. For any other inner call, (b) already models it as the
    correctly rounded double, as for an inline composite.
  - An inner non-integer result for an integer parameter stays skipped (decision 0297).
- Code: `eval_math.py` (the case `Workbook` and `formulas` cache), `composite.py` (`compute`), and the fit lab's
  `parse_arg`/`arithmetic` in tools/src/rustsheet_tools/fitlab/sweep.py. Regenerate only the affected sidecars
  (`rustsheet-accuracy math --function`, T-4483). t3444 takes about 40 minutes (T-3741).

## Acceptance criteria
- [ ] A reference to a formula cell whose formula is a call with a Gate A definition is read as a composite inner
      node. Unit tests in tools/tests cover the t3444 shape (`BINOM.DIST(A1,...)` and `BINOM.DIST(A1-1,...)` over
      `BINOM.INV`), a chain of two such cells, a cycle (no value), and an inner non-integer result for an integer
      parameter (skipped).
- [ ] SPEC-0029 "Math references for eval cells" and the eval_math.py docstring state the rule. They agree with
      decision 0297.
- [ ] The t3444 sidecar, and any other sidecar that gains values, is regenerated. `make eval-math-check` is current.
- [ ] After `make eval`, each of the 15 cells above is in report.json `r1c_left_out` as `phase1b` (Excel and the
      engine within 1e-9 of the math), or the Outcome says why not. The Outcome gives the R1(c) counts before and
      after for probes and probes-numeric, and lists the cells in every other case that gained a value.
- [ ] No eval/baseline file changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST's cdf math convergence at a large n (T-4782).
- Fitting Excel's small-p cdf (T-3961) or the bit-level cells (T-4784).
