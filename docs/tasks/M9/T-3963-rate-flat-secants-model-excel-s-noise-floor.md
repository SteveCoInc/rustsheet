---
id: T-3963
title: "RATE flat secants: model Excel's noise-floor iterates, since no rule on the model's flat-secant state fits (after T-3850)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3850]
campaign: accuracy
---

## Goal
RATE's flat-secant cells come out as in Excel. That needs a model of Excel's own iterates in the noise floor,
because T-3850 showed that no rule on the model's state at the flat step does it.

## Context
- T-3850 (financial.md "RATE flat secants: no rule on the model's state separates Excel's restarts from its holds";
  docs/research/t3850/): calls whose model run reaches the same flat secant (same args, r0, r1 and f1), differing only
  in the guess and so in the step, get different Excel values in no step order.
  - On probes/t3850-rate-same-state (240 calls in 40 such groups, drawn blind), 25 groups get more than one Excel
    value. 12 are solid: A55:A60, A109:A114, A175:A180 and A229:A234 mix `#NUM!` with numbers, and 8 groups spread
    over more than 4 ulp. The other 13 differ by 1 to 4 ulp.
  - Differences of 4 ulp or less are under the reproducibility floor of decision 0112 and reviewer check 7 (R4.4), so
    they are not targets. A second fill on 2026-11-03 reproduced every bit, but one refill doesn't clear the floor.
  - Excel's value is the hold on 29 cells, the restart from r0 on 111, and neither on 100. Today's rule gets 55.
  - So Excel's own iterates differ from the model's before the flat step. The likely cause is the residual's
    arithmetic in the noise floor (term order, or how the power or the sum is rounded). It is not a rule applied at
    that step.
- A wider restart (to step 200, |f1| unbounded) gains 70 and loses 14 cells on the new probe, but on the fit set it
  gains 8 and loses 12. The sampling decides the count, so no widening is a rule.
- Candidates: reproduce Excel's residual bit for bit at noise-floor points (probe the residual itself, for example
  through a run of 1 step), then rerun the secant on it. Ranking candidates owes decision 0188 R4's hold-out and fresh
  sweep.
- Code: `rate_secant` and `rate_residual` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).
- Known failures owned (from T-3850, which owned them before it closed):
  - every cell of probes/t3698-rate-flat-restart that is off Excel;
  - every cell of probes/t3698-rate-flat-restart-fresh that is off Excel;
  - every cell of probes/t3906-rate-restart-oos that is off Excel;
  - every cell of probes/t3850-rate-same-state that is off Excel.
  - The cases are named here on lines with no cell address, so this task owns every cell of them. It overlaps with
    T-3424 (the noise floor's type) on the error-against-number cells and with T-3916 (the owner's ruling on T-3698's
    restart).

## Acceptance criteria
- [x] A probe or finding that pins Excel's residual (or another cause) at the noise floor of at least two of the
      12 solid t3850-rate-same-state groups (an error against a number, or more than 4 ulp apart). Splits of 4 ulp
      or less are not targets (decision 0112, R4.4).
- [x] A model that fits it, implemented with no RATE cell regressing, R4 hold-out and fresh sweep recorded; or a
      recorded finding that none on the evidenced inputs does.
- [x] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- The narrow-or-revert ruling on T-3698's restart (T-3916, needs owner).

## Outcome
- **Finding:** the cause is not Excel's residual but the secant's path. A step whose new residual equals the last
  one (f2 = f1) keeps the older iterate r0 and replaces r1, so the next secant runs on r0 and r2. That makes Excel's
  value depend on the iterate before r0, which is why T-3850's same-state groups split. financial.md "RATE keeps its
  older iterate when a step leaves the residual unchanged"; the older restart and no-rule entries are marked
  superseded/resolved.
- **Residual candidates** (x87-register residual or secant, libm `pow`, Excel's exp-ln power, two factor orders) each
  lose 360 to 920 of the 2,400 literal RATE oracle cells, so the twice-rounded residual stands.
- **Engine:** `rate_secant` keeps r0 on f2 = f1 and sends a zero step (r2 = r1) to the cap's test at once; T-3698's
  restart (`RATE_RESTART_*`, its FITTED tag and the `restart` parameter) is gone. A flat secant now only comes on the
  start pair, which still holds (T-3348).
- **Decision 0188 R4 (structural model, kind (c)):** no constant was fitted.
  - R4.1 hold-out (the fit lab's hash split, 30% test, 5-point margin; `docs/research/t3963/holdout.py`), as
    bit-exact / within 4 ulp / within 15 digits:
    - keep-r0: train 1,416 / 1,524 / 1,549 of 1,704; test 576 / 616 / 630 of 696. The test exact rate is 82.8%,
      against 83.1% on train.
    - The merged rule: train 1,357 / 1,498 / 1,521; test 557 / 610 / 617.
  - The candidates were first ranked on all 2,400 cells, hold-out side included. Ranked on train alone (bit-exact,
    the fit lab's order), keep-r0 is still first: 1,416, then the secant on the iterate before r0 at 1,402
    (1,528 / 1,552 within 4 ulp / 15 digits), then an unconditional restart at 1,394.
  - R4.2 fresh sweep: Gate A batch accuracy/RATE/r4-T-3963, 300 points of `rustsheet-accuracy generate RATE --batch
    r4-T-3963`, drawn and filled (excel-graph) after the freeze. All 300 are inside the envelope, bit-exact under both
    models: the table's RATE takes no guess, so the rule seldom fires. The batch stays in the gate.
  - Where the rule fires: probes/t3963-rate-keep-r0-fresh. It is 240 calls on the existing probe series' args with
    new guesses, kept where the two models differ (`gen_fresh.py`, committed before the fill). It has no math
    sidecar, so its check is against Excel alone:
    - keep-r0: bit-exact / 4 ulp / 15 digits 59 / 91 / 104 of 240, 8 of the wrong type;
    - the merged rule: 9 / 66 / 77, 21 of the wrong type.
    - Of the 8 keep-r0 misses (a number where Excel is `#NUM!`), 4 were the merged rule's `#NUM!` too: A59, A60, A73
      and A188. Only A188 ends on a zero step. Decision 0260 lists all 4.
- **Eval delta:** probes fixed 107, regressions 37 (decision 0260: 19 within 4 ulp, 13 beyond, 5 of the wrong type),
  new case t3963-rate-keep-r0-fresh 59/240. Literal RATE cells 1,914 -> 1,992 bit-exact.
- **Acceptance:** on the 12 solid t3850-rate-same-state groups (72 cells) the model gets 19 bit-exact, the merged
  rule 11. In A55:A60 and A229:A234 every numeric cell is now Excel's (5 of 6 each, from 2 and 3; the one `#NUM!`
  in each is still missed) and A187:A192 goes from 2 to 4; A114's `#NUM!` is now Excel's. So the cause is pinned on
  those groups (item 1), not on all 12. Item 2: implemented with the R4 record above, but 37 cells regress, so "no
  RATE cell regressing" is not met; decision 0260 allows them.
- **Follow-up:** T-3424 keeps the wrong-type cells (zero steps Excel errs on). The noise-only nper-1/2 residuals where
  Excel still restarts (t3906 A15, A64) are filed as a new task.
