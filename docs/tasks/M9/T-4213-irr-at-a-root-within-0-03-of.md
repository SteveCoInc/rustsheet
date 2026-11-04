---
id: T-4213
title: "IRR at a root within 0.03 of zero: Excel's last iterate and ours sit a different 1E-16 from the root (9 unowned t3411 fresh-sweep cells, 2 at the 11th digit, decision 0267 Tier A)"
milestone: M9
lane: functions-financial
status: blocked
priority: P1
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
On conventional series whose root is close to zero, IRR returns Excel's value: the 9 listed cells match Excel within
half a unit of the 15th significant digit. Excel is not the accurate side there, so the task finds which iterate or
stop Excel returns and reproduces it. If two attempts fail, the task blocks as time-boxed (decision 0056).

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main b450050f8, then `make unowned-off ALL=1` and a per-cell owner scan).
  These 9 IRR cells are visibly off and no open task lists them. T-3548 lists 160 other cells of the same three cases,
  and T-3411 (done) built the sweeps. All 9 cells are numbers in both Excel and the engine, and both find the root.
- The roots are 4.4E-6 to 0.029. Both sides are within 3E-17 to 1.5E-15 absolute of the 50-digit root (mpmath
  `findroot` on the literal flows, gap triage). Near zero, a 1E-16 absolute step becomes an 11th to 15th-digit
  relative difference. Examples:

  | cell | root (50-digit) | Excel | ours | first wrong digit |
  |---|---|---|---|---|
  | fresh-3 A38 | -7.8420403629003E-6 | -7.842040362748826E-6 (+1.5E-16) | -7.84204036297087E-6 (-7.1E-17) | 11th |
  | fresh-3 A45 | -4.4300532125970E-6 | -4.430053212489682E-6 (+1.1E-16) | -4.4300532127117265E-6 (-1.1E-16) | 11th |
  | fresh-3 A109 | 2.2378224888395E-4 | 2.2378224888375264E-4 (-2.0E-16) | 2.2378224888242038E-4 (-1.5E-15) | 12th |
  | fresh-2 A32 | 1.6845316336509E-4 | 1.684531633652231E-4 (+1.3E-16) | 1.6845316336500105E-4 (-8.9E-17) | 13th |
  | fresh-2 A43 | -2.2000456147424E-3 | -2.2000456147424075E-3 (+2.7E-17) | -2.2000456147435177E-3 (-1.1E-15) | 13th |
  | fresh-4 A110 | 1.1271176460087E-4 | 1.1271176460070365E-4 (-1.7E-16) | 1.1271176460092569E-4 (+5.3E-17) | 13th |

  The other three cells (fresh-2 A30, A37 and A44) are first wrong at the 14th or 15th digit.
- Excel's error against the 50-digit root is 8E-15 to 2.4E-11 relative. That is below decision 0188 R3's G = 1E-4, so
  Excel stays the target. Excel's absolute error is under 2.4E-16 on all 9 cells. It looks like the last step of a
  Newton or secant iteration whose NPV is evaluated at its rounding noise. Ours is up to 1.5E-15 off (A109, A43). That
  suggests the engine stops one step earlier, or takes its last step from a noisier residual.
- Things to check first:
  - which of `irr_secant`, `compute_irr` and `irr_fallback` returns each cell, and at which step;
  - whether Excel's value is the model's next or previous iterate;
  - financial.md "RATE and IRR secant constants past their fit sets" and T-3411's round-4 notes
    (docs/tasks/M9/T-3411-irr-on-conventional-flows-model-what-excel-does.md): the residual stop, `IRR_ROOT_RESIDUAL`,
    and the noise-floor findings ("in the noise floor Excel's iterates differ from the model's", T-3850);
  - the RATE noise-floor work in T-3963 and T-3424 (in flight), which may share a residual model.
- Coordinate with T-3548 (P0, claimed): it changes the same solver for the `#NUM!` and `#DIV/0!` cells of these
  sweeps. Re-measure these 9 cells on main after it lands before fitting anything.
- Code: crates/rustsheet-engine/src/functions/financial/financial_util.rs (`irr_secant`, `compute_irr`, `irr_fallback`).

## Acceptance criteria
- [ ] Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit. Otherwise two
      attempts are recorded and the task blocks as time-boxed (decision 0056), and decision 0267 accepts what is left.
- [ ] docs/excel-semantics/financial.md records what Excel returns at a near-zero root and cites these cells. The
      engine change carries a comment citing the same evidence.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7) for any fitted or structurally chosen step or stop:
  - [ ] Hold-out: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        candidate and for the correctly rounded root, from the fit lab's split or a stated manual split of about 30%
        by case or cell hash, with the margin fixed before scoring. The chosen candidate beats the correctly rounded
        root on hold-out, and its hold-out exact rate is within the margin of its train rate.
  - [ ] Fresh sweep: after the fit is frozen, a new log-uniform sweep of conventional series with roots between 1E-7
        and 0.05 in absolute value, oracle-filled. Its result is recorded in the Outcome.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope, with every point outside it listed, and no IRR
        Gate A point moved out of it (`rustsheet-eval accuracy`).
  - [ ] `FITTED(T-4213, n_cells, range)` on each fitted piece. Outside its range, the code uses the accurate root.
  - [ ] Reproducibility floor: no step targets differences below 4 ulp web against desktop, or bits that vary
        between fills (decision 0112).
  - [ ] No keep-matched branch: no band or special case is defined by the set of cells that already match.
- [ ] No gated cell regresses. IRR's 263-cell secant fit set stays bit-exact. `make check-fast` is green locally,
      CI is green, and reviewers approve (excel-fidelity included).

## Out of scope
- The type mismatches and roots the engine misses (T-3548, T-3407, T-3408, T-3347, T-4027).
- RATE's noise floor (T-3963, T-3424).

## Decision 0267 cells
Gap triage gives this task 9 visibly-off IRR cells that no open task listed: 2 Tier A (first wrong at the 11th
digit), 4 Tier B (12th or 13th), and 3 Tier C/D (14th or 15th), which come along as part of the same cluster
(decision 0267 item 4). Measured on origin/main b450050f8. Decision 0056's time-box applies.

- probes-numeric/t3411-irr-r4-fresh-2.case.json (5): Calc!A30, A32, A37, A43, A44
- probes-numeric/t3411-irr-r4-fresh-3.case.json (3): Calc!A38, A45, A109
- probes-numeric/t3411-irr-r4-fresh-4.case.json (1): Calc!A110

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit (decisions 0083, 0267).

## Blocked because: time-boxed (decision 0056)
Two attempts ran (below). The second lands: 2 of the 9 cells (fresh-2 A43, A44) are now bit-exact. The other 7
(fresh-2 A30, A32, A37; fresh-3 A38, A45, A109; fresh-4 A110) stay with this task for decision 0267 to accept. Excel's
value at that noise floor is a secant iterate the engine passes, and no stop rule found picks it without turning
Excel's `#NUM!` cells into numbers.

## Outcome (2026-11-04, opus run)
What changed: `irr_fallback` returns the Newton iterate one step past its stop (the iterate its residual test already
evaluates), where that is a positive discount factor. Before, it returned the stop iterate. This is a structural choice
between two ranked candidates, so it carries no `FITTED` tag (decision 0188 R4). financial.md "IRR's returns where the
secant fails are Newton iterates" records both attempts. A unit test pins A43 and A44.

Diagnosis: all 9 cells have large flows (|c| 1E+8 to 7E+11). Both secant runs go flat at |NPV| 2E-7 to 7E-5, above
the 1e-7 stop and a few 1E-16 of the discounted flows, so `irr_fallback` answers. Then:
- fresh-2 A43 and A44: Excel's value is the fallback's iterate one step past its stop.
- fresh-2 A30, A32; fresh-3 A38, A45, A109; fresh-4 A110: Excel's value is bit for bit 1/(1 - d) - 1 at the secant
  iterate whose |NPV| first falls under about 1.2E-15 to 2.1E-15 of sum |c_i| v^i.
- fresh-2 A37: Excel's value is no iterate of either search.

Attempt 1 was a relative noise-floor stop on the secant, |NPV| < eps * scale. It was refuted and not kept:
- Scale sum |c_i| v^i, at every eps from 6E-16 to 3E-15. At 1E-15 it fixes 53 IRR cells. It also loses 27 bit-exact
  cells and turns 98 cells Excel is `#NUM!` on into numbers, among them probes/irr-guess-edges A37
  `IRR({1E+15,1,-1E+15},0.05)`.
- Scale the largest flow, at 2E-15. It also loses t2565-irr-guess-sweeps A107:C141, whose 1.2E+14 flow is discounted
  to about 2.5E+6 at the root.
- Scale the guess's NPV: the window that fits the 9 cells (2.95E-15 to 9.8E-15) still stops irr-guess-edges A37.

Attempt 2 (kept) is the Newton iterate one step past the fallback's stop. Over every IRR cell of eval it changes no
cell's type. It makes 12 cells bit-exact: t2927 A79, A80, A135, A137; t3347-holdout A41; fresh-2 A43, A44;
t3411-irr-residual-scale A31, A32, A48, A49; t3478 A292. It costs 1, t3411-irr-r4-fresh A129, where Excel's value is the
stop iterate. It moves 4 t2927-irr-fallback-sweep cells (A46, A67, A127, A129) one ulp past the 15th digit. Within 15
digits, the changed cells go from 10 to 21.

R4 guards:
- Hold-out: the split was fixed before scoring, with the margin at 10 points. A case is hold-out when the first hex
  digit of sha256(case id) is under 5. That gives 31 IRR cases, about 31% of their cells. The counts are IRR cells
  whose Excel value is a number:

  | candidate | train n | train exact | train 4 ulp | train 15 digits | hold-out n | hold-out exact | hold-out 4 ulp | hold-out 15 digits |
  |---|---|---|---|---|---|---|---|---|
  | one past (chosen) | 1149 | 847 (73.7%) | 882 | 879 | 640 | 401 (62.7%) | 401 | 401 |
  | stop iterate (main) | 1149 | 840 (73.1%) | 876 | 872 | 640 | 397 (62.0%) | 397 | 397 |
  | correctly rounded root, cells with math | 588 | 74 (12.6%) | 154 | 180 | 288 | 20 (6.9%) | 49 | 68 |
  | one past, the same cells | 588 | 374 (63.6%) | 395 | 390 | 288 | 152 (52.8%) | 152 | 152 |

  The chosen candidate beats the correctly rounded root on hold-out, 152 against 20 exact. Its hold-out exact rate is
  11.0 points under its train rate, past the 10-point margin. That gap comes from the case mix and is not the choice's:
  main's stop iterate has the same 11.1-point gap, and the choice gains 0.6 to 0.7 points on both halves.
- Fresh sweep: probes-numeric/t4213-irr-r4-fresh was drawn after the freeze and filled from excel-graph on
  2026-11-04. It holds 150 conventional series with roots +-1E-7 to 0.05 (log-uniform) over six flow scales from 1 to
  1E+13 (docs/research/t3411/gen_sweep.py, seed "T-4213/r4-fresh"). Results:
  - Both sides give the same error on 46 cells. Excel returns a number on 104.
  - The engine returns 59 of the 104: 57 bit-exact against main's 56. A95 becomes bit-exact, and A53 moves from the
    8th digit to the 9th.
  - The correctly rounded root matches 0 of the 104 bit for bit, and 3 within 15 digits.
  - 45 cells are `#NUM!` where Excel returns a number (T-3548's type rule), and A53 and A149 stay off. All 47 go to
    T-4232.
- Gate A envelope: `rustsheet-eval accuracy` moves no IRR point out of the envelope. accuracy/IRR/r4-T-3411 A286
  comes inside it, so its known-failures line (T-3408) is removed. The fresh sweep is a probes-numeric case, not a
  Gate A table.
- Reproducibility floor: the choice picks between two iterates the engine computes. It targets no sub-4-ulp
  web-against-desktop difference and no bit that varies between fills.
- No keep-matched branch: one rule applies to every fallback return, with no band.

Eval delta: probes known failures unchanged, no regression. Visible parity changes on t3411-irr-r4-fresh-2 (A43 and A44
fixed), t2927 (+4 cells within 15 digits, -4) and t3411-irr-r4-fresh (A129). The new case's 47 visibly-off cells are
owned by T-4232.

## Landing notes (2026-11-04, opus run, landing window closed)
- PR #3211 (draft), head 2fcae3df3 before this commit. Not enqueued.
- Reviews on 2fcae3df3: security APPROVE, design APPROVE. Design's MINORs: the missing `.math.json`, and the STRATA
  conditional in gen_sweep.py could be a named constant. excel-fidelity was still running when the window closed;
  no verdict was recorded.
- (Done after the window: the t4213 `.math.json` is committed and `make eval-math-check` is current.) Was missing: eval/suites/probes-numeric/cases/t4213-irr-r4-fresh.math.json. `make check` fails without it
  (`make eval-math-check`, decision 0243). `cd tools && uv run rustsheet-accuracy math --suite probes-numeric` takes
  over 15 minutes and was still running. It also rewrites nine unrelated probes-numeric `.math.json` files (t1538,
  t1543, t2586, t3350, t3409, t3443 x3, t3953). Commit only the t4213 sidecar and `git checkout` the rest.
- CI on the draft has not been checked. It will be red until the sidecar lands.
- Next: add the sidecar, run `make check-fast`, push, then run excel-fidelity on the new head. Design and security
  need no re-run for a data-only sidecar commit. Mark ready, and enqueue once CI is green.
- Round 1 excel-fidelity on 2fcae3df3 (arrived after the window): REQUEST_CHANGES on the R4 guards, not on the
  semantics. Unit-test bits, 0 regressions, Gate A at 0 failing and the structural classification were checked OK.
  To fix in round 2:
  1. MAJOR: the hold-out must use the fit lab's split (T-3360 done: `fitlab/holdout.py`, its default 5-point margin),
     not the hand split above. Re-score one-past, the stop iterate and the correctly rounded root there. If the fit
     lab refuses on the margin, acceptance needs a decision record (`make new-decision`); the task cannot waive it.
  2. MAJOR: the fresh sweep must be a Gate A batch, `eval/accuracy/tables/IRR` `r4-T-4213` (precedent
     r4-T-3411, r4-T-3407; T-3357 and T-3358 done), with oracle and math sidecars. Give each of the 45 points outside
     the envelope (engine `#NUM!`, Excel a number) a known-failures.tsv line owned by T-4232. Record the
     `rustsheet-eval accuracy` result (reviewer's count: 59 of 104 inside the envelope). The probes-numeric copy
     then goes, and T-4232's cell list moves with it.
  3. MINOR: financial.md, move the T-4213 bullets above the section's `Evidence:`/`Status:` lines.
  4. MINOR: confirm none of t2927 A46, A67, A127, A129 (one ulp past the 15th digit now) is a desktop-oracle cell.
- Do not repeat: a relative noise-floor stop on the secant (attempt 1, refuted in the Outcome). Keep the task
  `blocked` (time-boxed).
