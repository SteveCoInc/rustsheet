---
id: T-4837
title: "Discriminating-probe designer: inputs where two IRR models disagree"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4834]
---

## Goal
Given two IRR models, a tool finds flows and guesses where they disagree in type or past the 15th digit, and writes a probe spec with each model's prediction per cell.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 0, item 3; method of T-3698 (cells chosen because two rules disagree).

## Acceptance criteria
- [x] `fitlab/irr_discriminate.py`: random draws over the plan's section 3 axes, then a local search on the flows,
      writing a spec case and a predictions file.
- [x] Unit test: for the base replica against the RATE driver it finds at least 20 disagreeing inputs.
- [x] `make check` green; reviewers approve.

## Out of scope
- Filling probes (T-4838).

## Outcome
- New `fitlab/irr_discriminate.py`. `draw` makes random inputs over plan.md section 3's axes: n from 2 to 120
  (21+ included), the sign pattern (conventional, a sign change in the middle, a positive first flow), absolute
  scale 1E-15 to 1E+15, an occasional flow 1E+5 to 1E+15 times the others, the root location through the
  outlay-to-inflow ratio, the guess (0.1, 0, a near or far value, -0.5), and the structure (level, noisy, growing,
  a zero run, trailing zeros). `search` does the random draws, then the neighbourhood of every disagreeing input,
  then a hill-climb on the flows (`climb`, raising `distance`) from the closest of the rest. `write` emits a
  pending spec case (one `=IRR({flows},guess)` per cell) and a `.predictions.json` with each model's value and
  terminal event per cell. Disagreement means a different type, or a different 15-significant-digit rendering.
  Inputs keep 15 significant digits and print in shortest round-trip form, so each literal reads back as the
  same double.
- Base replica (cap 200) against the RATE driver (hold + run on, cap 200), seed 1: 40 disagreeing inputs from 400
  draws in about 18 s. 33 are `#NUM!` against a number (flat, zero-step or pole events against RATE's hold or
  run on) and 7 are `#NUM!` against `#DIV/0!`.
- Unit tests (tests/test_fitlab_irr_discriminate.py): literal round trip, the disagreement test, the draw axes,
  and that base against RATE finds at least 20 distinct disagreeing inputs, with the case and predictions files.
  docs/architecture/fit-lab.md documents the tool. No case is committed; filling one is T-4838's.
- Eval delta: none (tools only).
