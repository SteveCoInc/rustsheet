---
id: T-3850
title: "RATE flat secants Excel restarts late or leaves in the noise floor (after T-3698)"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] A probe that separates late restarts from holds, oracle-filled, and a rule that fits it, or a recorded finding
      that no rule on the run's visible state does.
- [ ] Any rule is implemented with no RATE cell regressing; R4 hold-out and fresh sweep recorded.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- RATE's residual form (ruled out at nper 1 by T-3579).
