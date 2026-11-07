---
id: T-4362
title: "BINOM.DIST.RANGE at a tiny p: Excel's 6.999999999999857E-120 for BINOM.DIST.RANGE(7,1E-20,6) (gen-m9 9cca17438a7e6f8a A360, the last unowned M9 rate cell)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4361]
campaign: visible
---

## Goal
BINOM.DIST.RANGE returns Excel's double where p is tiny, starting with the one M9 rate cell no task owns.

## Context
- `scripts/exit_owners.py --milestone M9` after `make eval` on c5fdf6bd3 lists one unowned cell:
  `rate:BINOM.DIST.RANGE gen-m9/9cca17438a7e6f8a.case.json!Calc!A360 BINOM.DIST.RANGE(7,1E-20,6)`.
  Expected (Excel) 6.999999999999857E-120, actual 7.000000000000068E-120 (`numeric_close`, visibly off: they differ at
  the 14th significant digit). BINOM.DIST.RANGE is one of the 41 functions below Gate B's rate (1/129).
- 50-digit math: C(7,6)·(1E-20)^6·(1 - 1E-20) = 6.99999999999999999993E-120. Excel is about 2.0E-14 below it, the
  engine about 1.0E-14 above it. Excel's error is far below decision 0188 R3's 1e-4, so Excel's digits are the target
  (directive 1). An error of this size at a tiny p suggests a log-domain form (EXP of n·LN-type sums, or Loader's
  kernel with p·(1-p) handled through LN) rather than the direct power; the engine's current path for a single-point
  range is in `functions/statistical/` (BINOM.DIST.RANGE shares BINOM.DIST's pmf since T-2146, decision 0293).
- **Needs an oracle sweep**: one Excel point does not fix a form, so this task depends on T-4361.
- Related, not overlapping: T-4352 (Loader's stored chain for the interior pmf), T-3740 (RANGE past the mean).
- gen-m9 b6593d7dd13f2dac A69 (`BINOM.DIST.RANGE(1,1E-100,4,VSTACK(FORECAST.ETS(...)))`, #N/A against #NUM!) is the
  FORECAST.ETS family's (T-4207), not this task's.

## Acceptance criteria
- [ ] gen-m9 9cca17438a7e6f8a Calc!A360 matches Excel; this 1 known-failure line disappears from
      eval/baseline/gen-m9.txt with no new ones in any suite.
- [ ] The T-4361 training and hold-out cases: every cell within 15 significant digits of Excel's value; the Outcome
      gives bit-exact and within-15-digit counts on each.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7): hold-out counts reported separately from training;
      a fresh sweep drawn after the form is frozen and scored; the Gate A envelope result (`make eval` accuracy line)
      for the touched tables; any fitted constant or cut tagged `FITTED` with its evidenced range; the
      reproducibility floor stated; no branch that only keeps already-matching cells matched.
- [ ] statistics.md records the form Excel uses at a tiny p, citing A360 and the T-4361 cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST and NEGBINOM.DIST interior pmfs (T-4352); the cdfs (bratio port follow-ups).

## Visible-parity stage
`visible`: a numeric cell off Excel at the 14th digit where neither Excel nor the engine is correctly rounded, so
neither `semantics` nor `accuracy` applies (decision 0114). Target: A360 and every T-4361 cell within 15 significant
digits of Excel; bit-level residue moves to a `defer: phase-1b` follow-up.

## Outcome
2026-11-07, partial: A360 is bit-exact. The T-4361 sweep has not run, so the task stays in progress.
- Form (structural, no fitted constant or new cut): where either bd0 takes its log form at bd0's own x87 cut, the
  interior pmf's exponent is fl(fl(S - bd0_p) - bd0_q) - lf/2 (S the Stirling sum as a double), stored before the
  extended -lf/2 and EXP. `loader.rs` `LoaderExponent::{Probability, ScaledProbability}`, `Bd0Branch::store`;
  BETA.DIST's density and NEGBINOM.DIST keep the register chain. statistics.md "BINOM.DIST pmf: a log-form bd0
  stores the deviance subtractions".
- Storing only X = -bd0_p - bd0_q left A360 at 75 ulp; the Stirling sum must enter the stored chain.
- `make eval`: 10 cells fixed (8 gen-m9, 1 ironcalc-calc, 1 probes), 0 regressions.
- Remaining: score the T-4361 training and hold-out cases, the fresh sweep and the Gate A line (R4 guards). This also
  moves interior pmfs with a log-form bd0, which T-4352 owns; T-4352 should re-score its 15 cells on this form.
