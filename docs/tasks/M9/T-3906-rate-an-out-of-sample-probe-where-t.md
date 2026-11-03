---
id: T-3906
title: "RATE: an out-of-sample probe where T-3698's flat-secant restart fires (R4)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
T-3698's fitted restart rule (`RATE_RESTART_STEPS = 16`, the |f1| window 1.3e-7..1e-4) is tested on points the fit never
saw and where the restart actually fires. The rule is kept, narrowed, or reverted on that evidence.

## Context
- PR #2858 (T-3698) merged with the Excel-fidelity reviewer's MAJOR finding open (orphan re-review, 2026-11-03,
  PR comment):
  - The step bound and the |f1| cut were scored on every cell before the train/test split, so R4.1's hold-out is not blind.
  - The two sweeps drawn after the freeze fire the restart 0 times, so R4.2 says nothing about the fitted values:
    - Gate A RATE: 121 flat secants, 0 restarts;
    - probes-numeric/t3698-rate-r4-fresh: 102 flat secants, 0 restarts.
  - The rule fires only on cells of the fit set (21 in each t3698-rate-flat-restart probe).
- Fix the reviewer asked for:
  - draw fresh annuities and guesses after the freeze, filtered to model runs that reach a flat secant on distinct
    iterates near the rule's window;
  - oracle-fill them with the constants untouched;
  - report bit-exact / 4 ulp / 15 digits for the rule, the hold and the correctly rounded root.
- The reviewer's MINOR findings come along:
  - give the |f1| cut its own constant under the T-3698 FITTED tag instead of reusing `RATE_CAP_TOLERANCE`, and make
    decision 0255 agree;
  - `RATE_RESTART_F_MIN`'s comment: inside steps 3..16 the smallest evidenced |f1| is 1.67e-7, not 1.3e-7.
- Code: `rate_secant` in crates/rustsheet-engine/src/functions/financial/financial_util.rs; docs/excel-semantics/financial.md
  "RATE restarts once on an early flat secant".

## Acceptance criteria
- [x] A probes case of fresh, oracle-filled (excel-graph) RATE calls where the frozen rule restarts, with the
      draw's seed and filter recorded. Scored rule vs hold vs correctly rounded root.
- [x] The rule is kept if it beats the hold out of sample; otherwise narrowed or reverted, with no RATE cell regressing
      beyond what the evidence justifies, recorded in financial.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Late restarts and the noise floor (T-3850).

## Outcome
- **Probe.** probes/t3906-rate-restart-oos, 240 RATE calls, filled by excel-graph on 2026-11-03.
  - Drawn after T-3698 merged, from 1,000,000 fresh annuities. The seeds and ranges are in the case description;
    the generator was a scratch script and is not committed.
  - Filtered with a scratch test that runs the frozen rule and the hold (`rate_secant(.., false)`) on every call.
    15,850 calls give different results, and 240 were sampled from them.
  - The 50-digit roots come from mpmath `findroot` on the FV residual, seeded at Excel's value.
- **Scores.** Bit-exact / 4 ulp / 15 digits:

  | | bit-exact | 4 ulp | 15 digits |
  |---|---|---|---|
  | rule, of 240 | 48 | 84 | 103 |
  | hold, of 240 | 12 | 76 | 104 |
  | correctly rounded root, of the 232 numeric cells | 17 | 64 | 91 |

- **Verdict: the rule stays.** It beats the hold on the bit and ties it at 15 digits, so nothing is narrowed or
  reverted. On 180 of the 240 Excel matches neither path. They are noise-floor runs of large annuities, so the case is
  now T-3850's evidence (named there as a whole case).
- **Reviewer MINORs from the T-3698 orphan review, with no behavior change:**
  - The |f1| cut is now its own constant `RATE_RESTART_F_MAX` (1e-4), covered by T-3698's FITTED tag and no longer
    tied to `RATE_CAP_TOLERANCE`. No new tag; quality/fitted.json is unchanged.
  - `RATE_RESTART_F_MIN`'s comment now gives its real evidence: 1.27e-7 at step 58, and 1.67e-7 within steps 3..16.
  - Decision 0255 is left as recorded. Its "the cut reuses `RATE_CAP_TOLERANCE`" is superseded by the code comment
    and this Outcome.
- **financial.md.** "RATE restarts once on an early flat secant" now says the Gate A batch never fires the restart,
  and gains the out-of-sample bullet.
- **Eval delta.** No existing cell changes. The new case's off cells are new coverage, owned by T-3850.
