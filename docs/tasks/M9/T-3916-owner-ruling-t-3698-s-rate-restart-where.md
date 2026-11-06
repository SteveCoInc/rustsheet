---
id: T-3916
title: "Owner ruling: T-3698's RATE restart, where both narrowing and revert move noise-floor points out of the Gate A envelope"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: S
tier: opus
depends_on: [T-3906]
---

## Goal
A ruling on T-3698's flat-secant restart in RATE: keep it, narrow it or revert it. Every option moves some point of
probes/t3906-rate-restart-oos out of the Gate A envelope. Most of the points that move lie inside their root's own
rounding noise.

## Context
- T-3906 (probes/t3906-rate-restart-oos, 240 fresh calls where the rule fires) scored the restart out of sample.
  The rule is bit-exact on 48 and the hold (no restart) on 12. The rule has 207 points inside the envelope, the
  hold 204.
  - The rule moves A152 and A194 out (A194 becomes `#NUM!` where Excel returns a number).
  - The hold moves A15, A26, A64, A76 and A121 out.
  - Decision 0188 R2 as reviewer check 7 applies it: "A fit that moves a point out of the envelope fails this item".
- T-3906 gave T-3850 a binding criterion: narrow the rule so that no t3906 point leaves the envelope, or revert it
  under a regression decision record. T-3850's search (docs/research/t3850, 2026-11-03) found that neither is
  possible without some point leaving:
  - 48 narrowings (restart count, fall back to the hold on an error, last step, |f1| cut and floor): every one with 0
    regressions still moves a point out (A152, A194 or A204). Every one with no point leaving regresses 9 to 16
    existing bit-exact cells.
  - A revert loses T-3698's 22 bit-exact cells and moves the five points above out.
  - A152's root has a rounding-noise width of 7.7e-9 relative (eps·Σ|terms| / |f'(r)| / |r|), almost 8 times ε = 1e-9.
    The rule is 1.36e-9 off the math and the hold 7.9e-10, so which side of ε it lands is chance. With roots from the
    decimal literals the point that leaves is A144 instead. Excel itself is 1.4e-8 off the math at A144.
  - A194 is not noise: the restarted run meets a second flat secant at step 92 with |f| 1.65e-4 (above the cap's
    1e-4) and is `#NUM!`. Falling back to the hold there fixes A194 but loses A26, where Excel and the restart are
    `#NUM!` and the hold is a number.
- R4.1, firing cells only: on the fit set's train side the rule is bit-exact on 17 of 40 (hold 0), on its test side 5
  of 14 (hold 0), and out of sample on t3906 48 of 240 (hold 12). The out-of-sample rate is about half the train
  rate, mostly from the large-annuity noise floor the fit set lacks (T-3850's notes).
- How decision 0188 applies here is the owner's call (decision 0188 is an owner decision). This task waits on that.

## Acceptance criteria
- [x] The owner's choice is recorded (a decision record when it changes how R2 applies), and the rule is kept, reverted
      or narrowed to match.
- [x] T-3850's ruling-dependent text and financial.md "RATE restarts once on an early flat secant" match the ruling.
- [x] `make check` green; reviewers approve.

## Out of scope
- Late restarts and noise-floor values (T-3850).

## Outcome
- Owner ruling 2026-11-04 (option 3), recorded as decision 0265. When a fit is checked on moving a point out of
  the envelope, a point whose root's rounding-noise width exceeds ε is judged against Excel's 15 digits only, and
  the task reports that width. The `make check` gate (SPEC-0029 `envelope`) is unchanged.
- Re-scored: A152 is outside under both the rule and the hold, so it no longer counts against the rule. A194, where
  the rule returns `#NUM!` and Excel a number, is accepted, and T-3963 owns it. T-3698's restart stays as merged, with
  no engine change.
- Updated: decision 0188 R2 (a note), reviewer-excel-fidelity check 7, financial.md, T-3850, T-3963,
  docs/research/t3850.
- Eval delta: none (docs only).
