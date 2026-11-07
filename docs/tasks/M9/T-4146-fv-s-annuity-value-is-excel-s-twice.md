---
id: T-4146
title: "FV's annuity value is Excel's twice-rounded form (probes/t3424-rate-noise-trace A132)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
FV gives Excel's bits where its plain-double evaluation and Excel's twice-rounded operators differ, starting with
probes/t3424-rate-noise-trace A132, which is the only literal FV oracle cell where the two forms differ today.

## Context
- T-3424 read Excel's FV at 355 secant iterates of the model's RATE run (its trace probe, excel-graph, 2026-11-04).
  Excel's FV equals -((pv·P) + (pmt·(k·((P-1)/r)))) with each `+`, `*` and `/` Excel's twice-rounded operator on
  all 355 cells. That is the model's (engine's) RATE residual without + fv (`annuity_factor_rounded` in
  crates/rustsheet-engine/src/functions/financial/annuity.rs).
- The engine's FV (`compute_future_value`, same file) evaluates the same expression in plain doubles
  (`annuity_factor`). Across every literal `=FV(` oracle cell of every suite (587 numeric cells), the twice-rounded
  form is bit-exact on 587 and the plain form on 586. The one cell where they differ is A132:
  `=FV((2.88607473601363e-05+1.01643953670516e-20),321,-2890484105.96678,934408648396.347,1)`, Excel
  -10935378180.042114, engine -10935378180.042236 (scorer: docs/research/t3424/fv_forms.py).
- One discriminating cell is not enough to change the form: choosing between two structural forms owes decision
  0188 R4's hold-out and fresh sweep (reviewer-excel-fidelity check 7). Draw a probe where the two forms differ
  (large |pv|·P and |pmt|·A of opposite sign, so the sum cancels; type 0 and 1; nper 1 to 400), fill it, and score.
- PV and NPER share `annuity_factor` (annuity.rs, PV at line ~763): check whether their oracle cells say the same.

## Acceptance criteria
- [x] A probe of at least 100 FV cells where the two forms differ, filled on excel-graph, with the form chosen on
      a hash-split train half and confirmed on the other half and on a fresh sweep (R4).
- [x] FV (and PV, if its evidence agrees) uses the winning form; A132 passes, or the evidence says otherwise and
      financial.md records it.
- [x] No FV, PV or RATE cell regresses; `make check` green; reviewers approve.

## Out of scope
- RATE's flat-secant outcome (T-3963).

Known failures owned: probes/t3424-rate-noise-trace A132.

## Outcome
- **Finding** (financial.md "FV and PV round each operation twice"): FV's and PV's `+`, `*` and `/` are Excel's
  twice-rounded operators. The form is a structural choice between two candidates (decision 0188 R4, reviewer check
  7 kind (c)); no constant was fitted, so no `FITTED` tag.
- **R4.1 hold-out:** probes/t4146-annuity-twice-rounded, 240 calls drawn by docs/research/t4146/gen_probe.py where
  the forms differ, committed before the fill (excel-graph, 2026-11-04). The scorer (`score.py`: the fit lab's hash
  split, margin 5 points, the correctly rounded candidate) was committed before scoring. Twice-rounded bit-exact:
  FV train 84/84, test 36/36; PV train 87/87, test 33/33. Plain doubles 0 on every side; correctly rounded 0. It
  beats the correctly rounded candidate on test and the train-test gap is 0.
- **R4.2 fresh sweep:** probes/t4146-annuity-twice-fresh, 120 calls on a new seed, drawn and committed after the
  freeze and before its fill: twice-rounded 120/120, plain 0, correctly rounded 0.
- **Gate A:** `rustsheet-eval accuracy` 0 failing points.
- **Engine:** `compute_future_value` and `fn_pv` use `annuity_factor_rounded` and Excel's `+`, `*`, `/`
  (annuity.rs); the once-rounded `annuity_factor` is gone (no other caller). NPER does not use the factor.
- **Eval delta:** probes fixed 1 (t3424-rate-noise-trace A132), regressions 0; two new cases, 360 of 360 passing.
- **Precision of the record (T-4244, from the #3226 fidelity review):** on the hold-out, bit-exact and within 4 ulp
  are the same for each candidate: twice-rounded 240/240 and 240/240; correctly rounded 0 and 0; plain doubles 0
  bit-exact, within 4 ulp only for PV (21/87 train, 8/33 test). The fresh sweep is 120/120 inside the envelope as a
  probes case (no FV/PV Gate A table exists). Both sweeps hold cancelling integer-nper calls only; T-4244's
  probes/t4244-fv-pv-nonint-nper extends the form to non-integer nper (80/80).
