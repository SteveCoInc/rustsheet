---
id: T-3850
title: "RATE flat secants Excel restarts late or leaves in the noise floor (after T-3698)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
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

Review follow-ups of T-3698 (PR #2858, fidelity and design MINORs, carried here):
- The restart's |f1| cut reuses `RATE_CAP_TOLERANCE` (T-2689's FITTED cap), so refitting that tolerance would move
  the restart too. Give the restart its own constant, or state the coupling, and make decision 0255 and the T-3698
  Outcome classify the cut the same way.
- `RATE_RESTART_F_MIN = 1.3e-7` comes from flat secants at step 46 and later (t3698-rate-flat-restart A43, A46, A47,
  A51-A54), outside the 3..16 band. The smallest |f1| evidenced inside the band is 1.7e-7 (t3579-rate-residual-nper1 A1).
- financial.md "RATE restarts once on an early flat secant" should list the misses inside the rule's region:
  t3698-rate-flat-restart A9 (|f1| 8.57e-5, step 3; Excel 0.002094170434467432 matches neither path), A17, A21, A25 and
  -fresh A22 (Excel `#NUM!`, both paths a number).
- Optional (design): move the restart constants to module level, next to `RATE_CAP_TOLERANCE`.

## Acceptance criteria
- [ ] A blind fresh probe that targets flat secants, drawn and filled before any rescoring, checks the restart rule out
      of sample (the T-3698 fresh sweeps hardly ever reach a flat secant).
- [ ] A probe that separates late restarts from holds, oracle-filled, and a rule that fits it, or a recorded finding
      that no rule on the run's visible state does.
- [ ] Any rule is implemented with no RATE cell regressing; R4 hold-out and fresh sweep recorded.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- RATE's residual form (ruled out at nper 1 by T-3579).
