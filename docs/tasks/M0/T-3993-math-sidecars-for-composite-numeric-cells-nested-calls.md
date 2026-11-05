---
id: T-3993
title: "Math sidecars for composite numeric cells: nested calls and arithmetic between calls of Gate A-defined functions (F.INV(F.DIST()), BETA.DIST(BETA.INV()), BINOM.DIST differences; LN(GAMMA()) once GAMMA has a definition)"
milestone: M0
lane: harness
status: in-progress
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

## Landing notes
Session 2026-11-05 (opus worker) ran out of its window. The branch head before these notes is 3de72c18f. No PR
was enqueued, and no reviewer has run.
- **Done:**
  - `tools/src/rustsheet_tools/accuracy/composite.py` parses composites. A call-free part is read by the fit lab's
    `arithmetic`/`parse_arg`.
  - It computes option (b), the default: the outermost node's math at the inner doubles, with inner `+ - * /` under
    Excel's X87 operator model. It also computes option (a), the exact composition, with `rounded=False`.
  - `eval_math.candidates` sends a formula to composites when it is not one call, or when its single call has an
    unparsed (nested) argument.
  - `tables.compute_value` is the shared dps-60/80 check, and `sweep.ARITH_TOKEN`/`MAX_NESTING` are now public.
  - There are unit tests in `tools/tests/test_accuracy_eval_math.py`, and 603 accuracy and fitlab tests pass.
  - The SPEC-0029 bullet is amended. Decision 0297 is drafted with `Status: proposed`, and its score counts are
    still TODO.
- **Sidecars, partly regenerated:**
  - The script used, which writes only the cases that hold a composite candidate, was
    `uv run python <regen_composites.py> probes-numeric probes`. It loops over `eval_math._spec_entries(suite)` and,
    for a case with any `Candidate.tree`, calls `case_math` and writes `sidecar_bytes`.
  - Every probes-numeric case with a composite is committed.
  - Probes stopped at `probes/t3444-binom-range-inv-large-n`. It has 84 BINOM.DIST composites at n = 1e9, each slow
    at 80 dps, and they ran past 10 minutes on one worker. That case and any later probes cases still need
    regenerating. Run it alone, in the background with a long timeout.
  - A whole-suite `rustsheet-accuracy math --suite probes-numeric` takes over 30 minutes; it spends it on single
    calls, mostly t3960.
- **Check before going on:**
  - The probes-numeric diff also deletes about 17k lines. Rewriting a whole case brings in generator drift on its
    one-call cells, such as old `"not converged or too slow"` skip entries that are now values. Confirm it is drift
    only, or keep the old entries.
  - A killed `ProcessPoolExecutor` leaves its spawn workers running, so `pkill` them as well.
- **Next:**
  1. Finish the probes regeneration, then run `make eval-math-check`.
  2. Run `docs/research/t3993/score.py` from the repository root after `make build`. It reads (b) from the
     sidecars, computes (a), and writes `cells.tsv`. Fill decision 0297's counts in from it and set the decision
     to accepted.
  3. Run `make eval`. The R1(c) line before was: "ironcalc-calc 26 counted, 0 Phase 1b evidence, 0 R3 passing, 0
     M14-owned; probes 959 counted, 326 Phase 1b evidence, 0 R3 passing, 16 M14-owned; probes-numeric 2581
     counted, 2925 Phase 1b evidence, 66 R3 passing, 0 M14-owned". `make unowned-off ALL=1` before showed 0 cells
     with no open owner (0 deferred).
  4. Write the Outcome, then `make check-fast`, then review.
