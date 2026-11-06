---
id: T-4495
title: "RATE past -1 at a non-integer nper and a -0 residual at the guess: probe the two solver rules financial.md leaves UNVERIFIED, and match"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence settles two RATE solver rules that financial.md leaves UNVERIFIED, and the engine
matches it. The first is what happens when a secant iterate lands at or below -1 with a non-integer
nper. The second is whether a -0 residual at the guess can occur and, if it can, which way it steps.
Both notes then read VERIFIED, or the UNVERIFIED wording is replaced by the evidence.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 4bd55c6c1). No visibly-off cell is unowned, so
  this comes from the semantics notes. No open task covers either rule: T-4431 owns the flat secant on
  a tiny or constant residual, and T-3424, T-3591 and T-3607 own noise-floor bits.
- financial.md "RATE holds on a flat secant and runs on below -1" (T-3348): "An iterate at or below
  -1 does not end the run: the residual is evaluated there (Excel's `^` takes a negative base at an
  integer nper) and the secant carries on. ... UNVERIFIED: an iterate below -1 at a non-integer nper.
  The engine's `^` errs on the negative base there and the run is `#NUM!`; Excel is unprobed."
  - Evidence for integer nper: probes/t3348-rate-below-minus-one has 50 runs whose iterate leaves
    -1, and the engine is bit-exact on 50/50 of them. For example, `RATE(3,3735.938842325,
    -6629.520726773,4839.177771065,1)` = 1.2149215363749337. The case was found at
    probes-numeric/t3348-rate-gate-a-sweep A171 (first step at -1.48).
  - Hypotheses to separate: (a) the run is `#NUM!` as soon as the residual cannot be evaluated (the
    engine today); (b) Excel's internal power returns NaN or a value for the negative base (for
    example |1+r|^n, or the real part), and the run either goes on or fails later; (c) Excel's power
    truncates or rounds nper to an integer inside the solver only.
- financial.md "RATE is a secant on the FV residual" (Status line) and "RATE steps up from a zero
  residual" (T-3876): "a -0 residual at the guess (a +0 one steps up ...; no probed one is -0)".
  The engine tests `f0 > 0.0`, so a -0 residual steps up as +0 does.
  - First decide reachability from the verified residual order (f(0) is `pv + pmt*nper + fv` in
    plain doubles, and the general form is in the same entry). An IEEE sum is -0 only when every
    addend is -0, and an omitted fv adds +0. If no RATE input can give -0, say so in financial.md and
    drop the note. No probe is needed in that case.
- Method (excel-probe skill): build the inputs with the engine's own model replay. The fit lab's RATE
  kernels (`tools/src/rustsheet_tools/fitlab`, `rate_candidates`) and the T-3348 scratch replays can
  find runs whose first or a later iterate goes below -1 at nper values such as 2.5, 3.7 and 10.25.
  Use guesses on both sides and pmt/pv/fv scaled as in t3348-rate-below-minus-one. For each run,
  commit before the fill what each hypothesis predicts (the value or the error). Include integer-nper
  twins of the same runs as controls, which must keep matching. A run that comes back above -1 under
  hypothesis (b) and converges is the most informative cell.
- Code: `rate_secant` and `rate_residual` in crates/rustsheet-engine/src/functions/financial/financial_util.rs
  (`annuity_power` errs on the negative base).

## Acceptance criteria
- [ ] A probes case (for example probes/t4495-rate-below-minus-one-fractional-nper) of at least 30
      runs that reach an iterate at or below -1 with a non-integer nper, plus their integer-nper
      controls, filled by excel-graph. The hypothesis predictions are committed in the case
      description or docs/research/t4495 before the fill.
- [ ] The -0 question is settled: either a cell that reaches a -0 residual at the guess, filled, or a
      financial.md paragraph that shows from the verified residual order that no input reaches one.
- [ ] The engine matches every new cell's type, and its value where the run converges. If a value
      needs a new rule, only the choice among the stated hypotheses is made on the cells. If that
      choice is a model (decision 0188 R4 kind (c)), the Outcome shows the hold-out and the fresh
      sweep: a second seeded set of runs drawn after the rule is frozen, its counts, and the Gate A
      envelope result on the RATE table.
- [ ] No RATE cell in any suite regresses. The t3348 cases and the m9-fin-irr-rate cells stay
      bit-exact (`make eval`). Known failures in eval/baseline/gen-m9.txt and probes.txt only shrink.
- [ ] financial.md: both UNVERIFIED notes are replaced with the evidence. Any further quirk gets its
      own entry citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The flat secant on a tiny or constant residual (T-4431) and the noise-floor last bits (T-3424,
  T-3591, T-3607).
- RATE's step constants and cap (fitted by T-2005 and T-2689). Do not refit them here.
