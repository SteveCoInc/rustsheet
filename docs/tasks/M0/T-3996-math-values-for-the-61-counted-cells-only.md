---
id: T-3996
title: "Math values for the 61 counted cells only defer: tasks own: F.DIST.RT and FDIST, the legacy CHIDIST and POISSON subnormal tail, CHISQ.DIST.RT there, GAMMALN and LN(GAMMA(x))"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3798]
---

## Goal
The 61 visibly-off cells below get a math value in their case's math sidecar, so decision 0188 R1(c) reads them
against the math instead of counting them for having none. Until then this task is their non-deferred owner, and
`scripts/phase1_exit.py`'s `owners` line no longer reports them.

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main b6ab50e5): `scripts/exit_owners.py --all-off` lists 61 counted
  cells, all `(deferred)`: only `defer: phase-1b` tasks own them, so `scripts/phase1_exit.py` says
  `owners  not met  61 visibly-off cells with no open owner (61 owned only by a deferred task)`. Every one is a
  last-digit miss (relative difference at most 1.2E-13, at t3638's values near 1E-300). It is
  counted only because it has no math value: five of the seven cases have no `.math.json`, and the other two (t2220,
  t1538) have one without these cells, because `tools/src/rustsheet_tools/accuracy/defs` defines none of F.DIST.RT,
  FDIST, CHIDIST, CHISQ.DIST.RT, POISSON, GAMMA or GAMMALN, and `eval_math` reads no nested call (T-3924 leaves out
  t3570's `LN(GAMMA(...))`).
- Examples (`make eval-show`):
  - probes-numeric/t2220-f-dist-sweep-d1-1 Calc!A501 `F.DIST.RT(4.18,1,7)`: Excel 0.08017804625424811, ours
    0.08017804625424818.
  - probes/t3638-subnormal-upper-tail Sheet1!A17 `CHISQ.DIST.RT(1412.0,7)`: Excel 9.773941430918599E-301, ours
    9.773941430918334E-301.
  - probes-numeric/t3570-r5-bpser-lg-aux Calc!E157 `LN(GAMMA(1.85))`: Excel -0.05592381301965709, ours
    -0.05592381301965721.
- Where the definitions come from, so this task does not build them twice:
  - T-3798 classifies F.DIST.RT, FDIST, CHIDIST, POISSON, GAMMA and GAMMALN and files their table tasks (the legacy-alias
    ruling decides how FDIST, CHIDIST and POISSON reuse the modern definitions). This task depends on it.
  - T-3925 builds CHISQ.DIST.RT (and leaves CHIDIST to T-3798's ruling).
  - The nested `LN(GAMMA(x))` form is this task's own work: `eval_math` gives a one-argument outer function over one
    inner call the math of the composition, where both have a definition. Reuse the definitions and
    `fitlab.sweep.parse_arg` (directive 3); no second evaluator.
- The value owners stay the deferred tasks (T-3640 for t3638; T-2221 and T-3199 for m9-f-dist-rt and m9-legacy-fdist;
  T-2221 and T-3474 for t2220; T-3474 and T-3592 for t1538; T-2122, T-3042 and T-3331 for t2464; T-2122 and T-3937
  for t3570). This task owns the cells because what keeps them counted is the missing math value, not the engine
  (the T-3924 pattern, decision 0245).
- Cells:
  - probes/t3638-subnormal-upper-tail.case.json (7): Sheet1!A17, A18, A23, A24, A36, A39, A42
  - probes-numeric/m9-f-dist-rt.case.json (1): Calc!A16
  - probes-numeric/m9-legacy-fdist.case.json (2): Calc!A55, A56
  - probes-numeric/t1538-beta-dist-at-excel-inv.case.json (1): Calc!A15
  - probes-numeric/t2220-f-dist-sweep-d1-1.case.json (4): Calc!A501, A768, A861, A867
  - probes-numeric/t2464-r2-bpser-small-gl.case.json (1): Calc!A3188
  - probes-numeric/t3570-r5-bpser-lg-aux.case.json (45): Calc!E130, E131, E132, E148, E149, E150, E151, E152, E153, E154, E155, E157, E158, E159, E168, E298, E299, E300, E316, E317, E318, E319, E320, E321, E322, E323, E325, E326, E327, E336, E466, E467, E468, E484, E485, E486, E487, E488, E489, E490, E491, E493, E494, E495, E504

## Acceptance criteria
- [ ] `eval_math` gives `=LN(GAMMA(x))` (an outer one-argument function over one inner call, both defined) the math
      of the composition; a unit test covers it and a nested form it leaves skipped.
- [ ] The math sidecars of the seven cases above are regenerated and committed (new `.math.json` files where none
      exist), and `make eval-math-check` is current.
- [ ] Every cell above has a math value, or the Outcome names the definition still missing and its open task.
- [ ] The Outcome quotes `scripts/phase1_exit.py`'s `owners` line and the `(deferred)` count of
      `scripts/exit_owners.py --all-off` before and after. A cell that is outside Gate A's envelope once it has a math
      value is named with its owner task, filed without `defer:` if none exists.
- [ ] `make eval-baseline` records nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (the deferred value owners above).
- Table and definition work T-3798's filed tasks and T-3925 own; arithmetic-argument and scaled forms (T-3924);
  non-integer df (T-3923).
