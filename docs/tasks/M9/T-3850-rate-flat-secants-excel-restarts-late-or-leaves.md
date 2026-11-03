---
id: T-3850
title: "RATE flat secants Excel restarts late or leaves in the noise floor (after T-3698)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3698]
campaign: accuracy
---

## Goal
A rule for the RATE flat secants that T-3698's restart rule leaves to the hold, or to neither path: late restarts
Excel takes, and noise-floor runs where Excel's value is neither the hold nor the restart.

## Context
- T-3698 (financial.md "RATE restarts once on an early flat secant") restarts once from the older iterate when the
  flat secant is on distinct iterates at step 16 or earlier with |f1| < 1e-4. It gains 22 of 1,573 RATE cells and
  loses none.
- Excel also restarts late on some series (probes/t3698-rate-flat-restart A35 at step 27, A39 at step 33, A42 at
  step 41), while it holds early on others (A28 at step 18). On many cells Excel's value is neither path: its
  iterates leave the model's in the noise floor (financial.md "RATE holds on a flat secant", the T-3389 notes).
- Candidates the T-3698 probes did not separate: a restart count within a run, a bound relative to the residual's
  terms instead of the step, and a restart point other than r0.
- Code: `rate_secant` (crates/rustsheet-engine/src/functions/financial/financial_util.rs). A rule chosen by scoring
  owes decision 0188 R4's hold-out and fresh sweep.
- Known failures owned: all cells of probes/t3698-rate-flat-restart and probes/t3698-rate-flat-restart-fresh that
  are off Excel.
  - The cases are named here on lines with no cell address, so this task owns every cell of them.
  - Overlaps with T-3424 (noise-floor type) on the error-against-number cells.
- Also owned: every cell of probes/t3906-rate-restart-oos that is off Excel. It holds 240 fresh flat-secant calls
  (T-3906, 2026-11-03), and on 180 of them Excel matches neither the hold nor the restart. They are mostly noise-floor
  runs of large annuities, so they are this task's evidence.
  - The case is named on a line with no cell address, so this task owns every cell of it.
- R4 decision owed (T-3906): on that case T-3698's rule moves A152 and A194 out of the Gate A envelope (A194 becomes
  `#NUM!`), which fails the Gate A envelope item (decision 0188 R2, reviewer check 7), though it brings five other points in.
  - Narrow the rule so that neither point leaves, or revert it.
  - Also compute the train-side bit-exact rate on firing cells only (R4.1), to compare with this probe's 48 of 240.
- Findings of the first pass (2026-11-03, docs/research/t3850; PR "T-3850 (partial)"):
  - No narrowing meets the criterion. Of 48 variants (restart count, fall back to the hold on an error, last step,
    |f1| cut and floor), each one with 0 regressions still moves a t3906 point out (A152, A194 or A204). Each one
    with no point leaving regresses 9 to 16 existing bit-exact cells.
  - A revert moves A15, A26, A64, A76 and A121 out and loses T-3698's 22 cells.
  - A152 lies within its root's rounding-noise width (7.7e-9 relative, almost 8 times ε). A194 is a real exit: the
    restarted run meets a second flat secant at step 92 with |f| 1.65e-4 and is `#NUM!`.
  - So the narrow-or-revert criterion moved to **T-3916**, blocked on an owner ruling ("Needs owner" in
    docs/status.md).
  - R4.1, firing cells only (the rule and the hold differ; the fit set without the R4 sweeps and t3906): on the train
    side of the hash split the rule is bit-exact on 17 of 40 and the hold on 0. On the test side it is 5 of 14 and 0.
    Out of sample (t3906) it is 48 of 240 (20%) and 12. So the out-of-sample rate is about half the train rate
    (42%): mostly the large-annuity noise floor the fit set lacks. T-3916 weighs this too.

Review follow-ups of T-3698 (PR #2858, fidelity and design MINORs, carried here):
- Done by T-3906 (`RATE_RESTART_F_MAX`, decision 0255 amendment): the restart's |f1| cut reused
  `RATE_CAP_TOLERANCE` (T-2689's FITTED cap), so refitting that tolerance would move the restart too. Give the
  restart its own constant, or state the coupling, and make decision 0255 and the T-3698 Outcome classify the cut
  the same way.
- Done by T-3906 (the constant's comment now cites it): `RATE_RESTART_F_MIN = 1.3e-7` comes from flat secants at
  step 46 and later (t3698-rate-flat-restart A43, A46, A47, A51-A54), outside the 3..16 band. The smallest |f1|
  evidenced inside the band is 1.7e-7 (t3579-rate-residual-nper1 A1).
- financial.md "RATE restarts once on an early flat secant" should list the misses inside the rule's region:
  t3698-rate-flat-restart A9 (|f1| 8.57e-5, step 3; Excel 0.002094170434467432 matches neither path), A17, A21, A25 and
  -fresh A22 (Excel `#NUM!`, both paths a number).
- Optional (design): move the restart constants to module level, next to `RATE_CAP_TOLERANCE`.

## Acceptance criteria
- [x] A blind fresh probe that targets flat secants, drawn and filled before any rescoring, checks the restart rule out
      of sample (the T-3698 fresh sweeps hardly ever reach a flat secant). (T-3906: probes/t3906-rate-restart-oos)
- [x] Moved to T-3916 (it was: narrow T-3698's restart so that no point of probes/t3906-rate-restart-oos leaves the
      Gate A envelope, or revert it under a regression decision record). Neither is possible without a point leaving
      (docs/research/t3850), so the choice waits on the owner's ruling.
- [x] A probe that separates late restarts from holds, oracle-filled, and a rule that fits it, or a recorded finding
      that no rule on the run's visible state does. (Finding: probes/t3850-rate-same-state, below.)
- [x] Any rule is implemented with no RATE cell regressing; R4 hold-out and fresh sweep recorded. (No rule: none
      fits, see the Outcome.)
- [x] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- RATE's residual form (ruled out at nper 1 by T-3579).

## Outcome
- The finding (financial.md "RATE flat secants: no rule on the model's state separates Excel's restarts from its
  holds"; docs/research/t3850/README.md, second pass): no rule on the run's visible state at the flat step (step,
  |f1|, |r1 - r0|, f1 relative to the terms) separates Excel's late restarts from its holds. Calls whose model run
  meets the same flat secant (same args, r0, r1 and f1) and differ only in the guess get different Excel values, in
  no step order.
  - In the older cases: 4 of the 9 repeated states (t3698-rate-flat-restart A15, A27, A33, A34; A16, A25, A42; A43,
    A47, A51, A54; A13, A48).
  - New blind probe probes/t3850-rate-same-state: 240 calls, 40 same-state groups at steps 6 to 200, drawn by
    `gen_late.py` and committed (6de64ea8) before the excel-graph fill. 25 groups get more than one Excel value.
    12 of them are solid (4 mix `#NUM!` with a number, 8 spread over more than 4 ulp). 13 differ by 1 to 4 ulp,
    under decision 0112's floor, although a second fill reproduced all 240 cells bit for bit. Excel is the hold on
    29, the restart on 111, neither on 100 (mostly last bits); T-3698's rule gets 55.
- No rule is implemented and the engine is unchanged. Wider restarts (K up to 200, |f1| cut up to none) were scored
  on the 2,160 older RATE oracle cells. The best gains 2 (under the 5-point margin), and K 200 gains 8 and loses 12.
  On the new probe it gains 70 and loses 14. That contradiction says the count follows the sampling, so no fit was
  picked and the R4 hold-out and sweep do not apply.
- Also recorded: the T-3698 review's misses inside the rule's region, in financial.md "RATE restarts once on an early
  flat secant" (A9, A17, A21, A25, -fresh A22).
- Follow-up: T-3963 (opus) models Excel's residual in the noise floor. It now owns the off cells of
  t3698-rate-flat-restart, -fresh, t3906-rate-restart-oos and t3850-rate-same-state. The narrow-or-revert ruling
  stays with T-3916 (needs owner).
- Eval delta: no fix and no regression. One new probe case (240 cells), of which 55 pass, added to the baseline as
  known failures.

