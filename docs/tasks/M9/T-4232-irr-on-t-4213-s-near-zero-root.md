---
id: T-4232
title: "IRR on T-4213's near-zero-root sweep: 45 cells #NUM! where Excel returns a root, and A53/A149 at the noise floor (decision 0267)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IRR gives Excel's type and value on the visibly-off cells of probes-numeric/t4213-irr-r4-fresh, T-4213's fresh sweep of
conventional series with roots 1E-7 to 0.05. If two attempts fail, the task blocks as time-boxed (decision 0056) and
decision 0267 accepts what is left.

## Context
- T-4213 drew the sweep after freezing its change (the fallback returns the Newton iterate one step past its stop) and
  filled it from excel-graph on 2026-11-04. Of the 150 cells, Excel and the engine give the same error on 46 and Excel
  returns a number on 104. The engine returns a number on 59 of those (57 bit-exact). The rest are visibly off:
  - 45 type mismatches. Excel returns a root and the engine is `#NUM!`: both secant runs fail and the fallback is
    `#NUM!`. This is T-3548's open type rule (blocked, time-boxed), "Excel errs on runs of 98 and 129 steps and
    returns runs of 182". Re-measure them against whatever T-3548's successor lands.
  - A53 `-1.1597865900991877E-7` (ours on the T-4213 branch -1.1597865923196338E-7, on main -1.1597868432300373E-7 (first wrong at the 8th digit), math -1.1597865918101396E-7): first wrong at the 9th
    digit. The root sits 1.2E-7 from zero, so the engine's error is 2.2E-16 absolute.
  - A149 `9.855957131432724E-5` (ours 9.85595713141052E-5): first wrong at the 12th digit, also a 2.2E-16 absolute
    step.
- financial.md "IRR's returns where the secant fails are Newton iterates" (the T-4213 bullets): at this noise floor,
  Excel's value is sometimes a secant iterate and sometimes a Newton iterate, and no stop on |NPV| fits. That bullet
  lists the refuted relative stops.
- Code: crates/rustsheet-engine/src/functions/financial/financial_util.rs (`irr_secant`, `irr_fallback`).

## Acceptance criteria
- [ ] Every cell in "Decision 0267 cells" has Excel's type and is within half a unit of its 15th significant digit,
      or two attempts are recorded and the task blocks as time-boxed (decision 0056).
- [ ] The R4 guards (decision 0188; reviewer-excel-fidelity check 7) for any fitted or structurally chosen step.
- [ ] No gated cell regresses. `make check` is green and reviewers approve.

## Out of scope
- RATE's noise floor (T-3963, T-3424).

## Decision 0267 cells
- probes-numeric/t4213-irr-r4-fresh.case.json (47): Calc!A2, A4, A6, A8, A16, A17, A20, A23, A26, A36, A38, A41, A43,
  A45, A47, A48, A49, A50, A51, A52, A53, A54, A56, A58, A60, A62, A63, A64, A65, A68, A69, A70, A72, A73, A90, A92, A93,
  A98, A99, A127, A132, A137, A138, A140, A143, A145, A149

## Attempt 1 (2026-11-07, opus run): a flat secant run that settled in NPV noise
Finding (excel-graph evidence, probes-numeric/t4213-irr-r4-fresh, filled 2026-11-04): on the large-flow strata the
secant from 0.1 goes flat (f1 = f0 after a nonzero step) in NPV's rounding noise, never reaching the 1e-7 residual
stop, and the fallback's 50-step Newton search gives up. Excel's value there is bit for bit one of the run's last
secant iterates. Whether Excel returns or is `#NUM!` is decided by the smallest |NPV| at an iterate reached by two
steps under 1e-7 in a row: all 31 Excel numbers among those flat runs have it at most 9.73E-5 (A90), all 42 Excel
`#NUM!` cells at least 1.12E-4 (A81). That is the bracket of `IRR_ROOT_RESIDUAL` (1E-4, T-3411), so no new
constant. Which iterate Excel returns is the last one on 23 of the 31, an earlier one (`d0`'s predecessor or the
other point of a two-point oscillation) on 8; the replica's NPV noise does not separate them (as T-4213 found).

Model (frozen and pushed at f85247120 "frozen before the fresh sweep", before either fresh sweep was drawn):
`irr_secant` returns `SecantStop::Settled(rate at d1)` on such a flat run within `IRR_ROOT_RESIDUAL_FLOWS`;
`compute_irr` returns it only where `irr_fallback` finds no root. Scored candidates (all IRR cells of eval, cells
bit-exact against main / newly off): secant-first, min-|NPV| iterate +71/-4; secant-first, last iterate +74/-6;
fallback-first, min-|NPV| +58/-1; **fallback-first, last iterate +59/-1 (chosen)**, plus 24 cells from `#NUM!` to
within 15 digits. The one cell lost: t3411-irr-r4-fresh-2 A39 (Excel `#NUM!`, settled |NPV| 6.5E-5). The two-step
guard was added after a one-step guard broke t3478-irr-r4-wide A58 (flows under 1E-5, a run that jumps to d = 4 and
back). On this task's case: 108 -> 121 of 150 bit-exact, type mismatches 39 -> 23.

Fresh sweeps (drawn after the freeze, filled from excel-graph 2026-11-07):
- probes-numeric/t4232-irr-r4-fresh (gen_sweep.py seed "T-4232/r4-fresh", T-4213's six near-zero strata): main
  101/150 bit-exact (42 type mismatches), the model 113/150 (19 type mismatches).
- Gate A accuracy/IRR/r4-T-4232 (300 points, `rustsheet-accuracy generate IRR --batch r4-T-4232`): main 13 points
  outside the envelope, the model 10 (A215, A229, A261, A279, A286 move in; none out). Outside with the model:
  A148, A175, A209, A236, A243, A245, A288, A291, A292, A298.
- Gate A overall: 44180 -> 44205 points in the envelope; 22 known-failures lines become stale (r4-T-3411 and
  r4-T-3407 IRR points now inside). **But two points move out**: accuracy/IRR/r4-T-3411 A296
  `IRR({-341718254096.523,120486762928.076,207737518998.484})` and r4-T-3407 A249, both Excel `#NUM!` where the
  model now returns the root. Reviewer check 7 fails a structural model that moves a point out, so this needs either a
  rule that keeps them or a regression decision record (decision 0082's precedent).
- Tried after the freeze and NOT kept (it would make the fresh sweeps training data): use the settled iterate only
  when the fallback ran out of steps, not when it converged and rejected its root. It keeps r4-T-3411 A296 but not
  r4-T-3407 A249 or fresh-2 A39. Patch kept out of the repo.

## Landing notes (2026-11-07 03:05 UTC, opus run, landing window closing)
- Branch agent/functions-financial/T-4232-irr-near-zero-root; draft PR #3825, head 820a8a987 plus this notes commit (CI expected red until the gaps below are closed). Not enqueued, no
  reviewer has run.
- Not done yet: the hold-out table (fit lab split, `tools/src/rustsheet_tools/fitlab/holdout.py`, margin 5 points)
  for the chosen candidate, main and the correctly rounded root; `known-failures.tsv` lines owned by T-4232 for the 10 r4-T-4232 points outside the envelope and removal
  of the 22 stale lines; `make eval-baseline` (CI's eval gate will report fresh-2 A39 as a regression); a
  financial.md entry under "IRR's returns where the secant fails are Newton iterates".
- Done after the notes: probes-numeric/t4232-irr-r4-fresh.math.json is committed (`rustsheet-accuracy math --suite
  probes-numeric --function IRR`, which also rewrote t3347-irr-conventional-sweep.math.json: reverted, the check
  passes on the committed one); `make eval-math-check` is current; a unit test pins t4213-irr-r4-fresh A54
  (`a_flat_secant_run_settled_in_npv_noise_returns_its_last_iterate`); fmt and clippy are clean.
- Next step: decide the two Gate A points moved out. Look for a rule in the flows that separates r4-T-3411 A296,
  r4-T-3407 A249 and fresh-2 A39 (all Excel `#NUM!`, settled under 1E-4) from the 31+ settled Excel numbers, on
  a fresh hold-out; if none, a regression decision record (`make new-decision`) for 3 lost cells against about 85
  gained, then the review panel (design, security, excel-fidelity).
- Do not repeat: secant-first ordering (loses 3-5 bit-exact cells where today's fallback already gives Excel's
  bits: t3411-irr-r4-fresh A129, fresh-2 A108, t3347-irr-conventional-sweep A100); a one-step convergence guard.
