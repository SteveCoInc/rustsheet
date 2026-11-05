---
id: T-4349
title: "CUMPRINC's short-range form choice: find what decides it where the leading exponent does not (T-3524 follow-up)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3524]
---

## Goal
CUMPRINC and CUMIPMT over a short range (at most 32 periods) choose between the geometric sum and the per-period
sum or closed form by a rule that generalizes to fresh cells, or `GEOMETRIC_MIN_LEAD` is shown to be the best
available rule on a fresh sweep.

## Context
- T-3524 re-validated `GEOMETRIC_MIN_LEAD` (0.3, functions/financial/cumulative.rs) on a fresh sweep
  (probes-numeric/t3524-cumprinc-lead-fresh-sweep) and on the old cells (fit lab `cumulative`, `cumulative-fresh`;
  builds from docs/research/t3524/*.patch). Without the cut (the geometric sum at every leading exponent) the engine
  is one test cell better on the old cells (147 vs 146 bit-exact) and 6 better on the fresh sweep (514 vs 508), but
  38 old cells go visibly off (tiny-rate interests: `CUMIPMT(1E-10,296,197540,13,19,1)`, gen-m9 b72d952c5f52cb4d
  A257, 33,554,432 ulps; probes/fin-cumipmt-tiny-rate-structure A1, A3, K3; fin-cumipmt-gate-edges A8, E2)
  against 20 it fixes, and on the fresh sweep 9 against 13. So the leading exponent alone does not decide which
  form is Excel's over a short range: below 0.095 the old cells favour the cut and the fresh sweep the geometric
  sum (fresh 46 vs 57 bit-exact), from 0.095 to 0.3 both favour the cut. The rate (cells below 1E-3 dominate the
  visible losses) or the type are candidate deciders. `docs/research/t3524/analyze.py` lists each cell either form
  wins, with its lead and ulps.
- Owns the visibly-off cells of probes-numeric/t3524-cumprinc-lead-fresh-sweep after T-3524 (61, nearly all
  CUMIPMT, whose interest carries an ulp of the principal and of PMT as hundreds of its own): over a short range
  S!B3, B9, B10, B19, B37, B45, B52, B54, B70, B71, B76, B77, B79, B85, B86, B101, B104, B106, B110, B119, B126, B128,
  B131, B132, B133, B148, B152, B154, B170, B172, B175, B177, B186, B190, B197, B206, B210, B214, B223, B226, B228,
  B233, B234, B241; over a long range S!A404, B258, B275, B287, B336, B346, B379, B385, B401, B415, B432, B434, B439,
  B456, B464, B497, B500. A cell this task's rule does not reach goes to a follow-up it files.
- A new cut or selector is a fit: decision 0188 R4 (hold-out split, a fresh sweep drawn after the freeze).

## Acceptance criteria
- [ ] A ranked comparison (fit lab, two or more engine builds) of candidate short-range selectors on the old and the
      T-3524 cells, with hold-out counts, and a fresh sweep drawn after the choice is frozen.
- [ ] The chosen rule beats `GEOMETRIC_MIN_LEAD` on test and on the fresh sweep with no visibly-off regression
      beyond decision 0083's allowance, or the Outcome shows no candidate does and the cut stays.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Long ranges (T-3524 removed their cut); type 0's underflowed principal (T-3141).
