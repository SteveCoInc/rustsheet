---
id: T-3993
title: "Math sidecars for composite numeric cells: nested calls and arithmetic between calls of Gate A-defined functions (F.INV(F.DIST()), BETA.DIST(BETA.INV()), BINOM.DIST differences; LN(GAMMA()) once GAMMA has a definition)"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A probe cell whose formula composes calls of functions that each have a Gate A definition gets a 50-digit math value
in its `<case>.math.json`. That covers a call nested in another call's arguments and arithmetic between call results.
R1(c) can then tell Phase 1b evidence (Excel and the engine both within 1e-9 of the math) from real visibly-off
cells, instead of counting every such cell as outside the envelope. Which math value a composite gets is a design
choice, recorded in a decision record before any sidecar is written.

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main eda15c023, excel-graph). SPEC-0029 "Math references for eval cells"
  and `tools/src/rustsheet_tools/accuracy/eval_math.py` give a math value only to a cell that is one call `=FUNC(args)`
  with literal or exact-input arguments. A nested call is "left unparsed" and gets none. A cell with no math value
  counts as outside the envelope (decision 0188 R1(c)), so every visibly-off composite cell counts in the M9 probe row
  (869 R1(c) cells of 1,190 visibly off).
- T-3924 gives math values to arithmetic over exact-input cells inside a call's arguments and to an exact outer
  power-of-two or -0 wrapper. Its Out of scope excludes "nested calls such as t3570's `LN(GAMMA(...))`" and arguments
  that read formula results (t3444's `BINOM.DIST(A1-1,...)` over BINOM.INV). T-3923 covers argument forms (truncated
  df, BETA bounds, RATE's guess). No open task covers composites.
- Counted visibly-off probe cells whose formula is a composite of defined functions (this run). The owner tasks keep
  their cells: this task changes only how R1(c) classifies them, so each case is named with one example cell.
  - `F.INV(F.DIST(x,d1,d2,TRUE),d1,d2)` and `BETA.INV(BETA.DIST(x,a,b,TRUE),a,b)`, 41 cells over three cases (owners
    T-3597, T-3923): probes-numeric t3443-basym-sweep Calc!A205, t3443-basym-hold Calc!A205 and t3443-basym-a Calc!A200.
  - `BETA.DIST(BETA.INV(p,a,b),a,b,TRUE)`, 14 cells (owners T-3562, T-3923): t3566-beta-inv-tiny-shape-and-range Calc!A93.
  - `BINOM.DIST(k1,n,p,TRUE)-BINOM.DIST(k2,n,p,TRUE)`, 22 cells (owners T-3740, T-3741):
    t3444-binom-range-inv-large-n Range!B91.
  Composites that wait for a definition of an inner or outer function:
  - `LN(GAMMA(x))`: 45 visibly-off cells in column E of T-3570's lg-aux probe (owners T-2122 and T-3937, both
    `defer: phase-1b`). They are 45 of the 60 cells that `make unowned-off ALL=1` lists as owned only by a deferred
    task. GAMMA's definition is T-3798's.
  - NORM.S.INV/NORM.S.DIST round trips, 23 cells (owner T-0925; definitions T-3783): t1746-norm-s-inv-central Calc!B9 and
    m9-norm-s-inv Calc!A469.
  - F.INV.RT(F.DIST.RT()) and FINV(F.DIST.RT()) in the three t3443 cases, 16 cells (definitions T-3798): t3443-basym-a Calc!A219.
  - ODDFYIELD and YIELD over `DATE` arguments, and IM* calls over `COMPLEX`, once those functions have definitions
    (the sweep reader already reads a `DATE` of literals; `COMPLEX` would need the same).
- mpmath at 50 digits, on the doubles Excel reads, shows the pattern R1(c) is meant to sort. In
  `LN(GAMMA(1.85))` Excel is -2.8E-15 relative off the exact composition and the engine -6.8E-16. That is Phase 1b
  evidence, but today it counts as visibly off.
- The design question. Which value is "the math" of a composite?
  - (a) The exact composition: F.INV(F.DIST(x)) is x.
  - (b) The outer function's math at the correctly rounded inner math value.
  - (c) The outer function's math at Excel's own inner double. This is not available to tools, which never read
    oracle values as inputs.
  Under (a), an ill-conditioned inverse amplifies the inner rounding past 1e-9 relative, even for a perfect double
  pipeline. Under (b), the inner rounding is part of the reference. Score (a) and (b) on the cells above before
  deciding, by how many land within 1e-9 for Excel and for the engine. Record the choice and the counts in a decision
  record (`make new-decision`) that amends SPEC-0029 "Math references for eval cells". A spec change needs a decision
  record (CLAUDE.md, directive 2).
- Code: `eval_math.py` (`single_call`, `candidates`, `LiteralBook`), `fitlab/sweep.py`'s reader, and the runner's
  sidecar reader (crates/rustsheet-eval/src/accuracy/math.rs), which matches on the formula text `f`.

## Acceptance criteria
- [ ] A decision record states the composite math (one of the options above, or another), with the per-option counts
      on the cells listed above, and amends SPEC-0029's "Math references for eval cells" bullet on what gets a value.
- [ ] `rustsheet-accuracy math` writes a value for each cell above whose functions all have definitions. A cell with
      an undefined function stays skipped and is listed under `skipped` with its reason. Unit tests in
      tools/tests/test_accuracy_eval_math.py cover a nested call, a difference of two calls, and a composite with an
      undefined inner function.
- [ ] Sidecars of the touched suites are regenerated and committed, and `make eval-math-check` is current.
- [ ] The Outcome quotes report.md's R1(c) line ("Visibly-off cells under decision 0188 R1(c)") before and after, and
      the `(deferred)` count of `make unowned-off ALL=1` before and after. Every cell that now has a math value and lies
      outside Gate A's envelope is named in the Outcome with its owner task.
- [ ] `make eval-baseline` records nothing (no engine change). `make check` green; reviewers approve.

## Out of scope
- Definitions for functions that lack one (GAMMA, F.DIST.RT, F.INV.RT, the NORM family, ODDFYIELD, YIELD, the IM*
  functions). Those are T-3798, T-3783 and their table tasks. Once a definition lands, its composites follow without
  further work.
- Arithmetic over exact-input cells and outer wrappers (T-3924). Argument forms (T-3923).
- Any engine change, and the owners of the cells (every listed cell keeps its owner task).
