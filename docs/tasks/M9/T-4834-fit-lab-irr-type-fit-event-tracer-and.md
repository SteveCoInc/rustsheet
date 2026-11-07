---
id: T-4834
title: "Fit lab: irr-type fit, event tracer and path-invariant transforms for IRR"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The fit lab scores every IRR candidate on result type (number, `#NUM!`, `#DIV/0!`) and bits over all IRR probe cases, with the hold-out split, and reports the event that ends each replica run.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 0, items 1, 2 and 4.
- docs/research/irr-2026-11-07/feasibility.py is the prototype (plain doubles); `kernels/irr_secant_d.py` has the
  exact twice-rounded replica; `kernels/irr_fallback.py` the fallback candidates.
- The fit lab scores IRR's secant (`irr`, 263 cells) and fallback (`irr-fallback`, 212) on separate small sets.

## Acceptance criteria
- [x] A fit `irr-type` in `fitlab/kernels/` over every IRR probe case (errors scored) with candidates for the base
      replica, the VB handlers and the RATE driver; its counts match feasibility.py's types within a few cells (the
      exact arithmetic may move noise-floor cells; the Outcome lists the moved ones).
- [x] Candidates expose `trace` and the terminal event and step per cell, in the JSON.
- [x] Transform helpers (x2^k scaling, negation, leading and trailing zero padding) with unit tests that x2^k keeps a
      scale-invariant replica's iterates bit-identical.
- [x] docs/architecture/fit-lab.md's shipped-fit table lists the fit.
- [x] `make check` green; reviewers approve.

## Out of scope
- Probe cases (T-4838).

## Outcome
- New fit `irr-type` (`fitlab/kernels/irr_driver.py`). It runs feasibility.py's eight candidates on the exact replica: the secant in d, with NPV as worksheet NPV's running product and every operator rounded twice (`irr_secant_d._npv_v_running`), plus the retry from 0.1. The candidates are the base replica (caps 50 and 200), VB's flat nudge and r-half/d-half pole clamps, and RATE's hold on flat, run on past d = 1 and capped return. It covers the 2,555 IRR cells of the 26 IRR probe cases whose flows and guess are numbers, error cells scored, with the hold-out split.
- Type score: a new `typed` count in `score.Score` and the JSON. It counts the cells whose outcome type is Excel's: a number for a number, the same error code for an error. On the exact replica every candidate's type count equals feasibility.py's plain-double count: base 1,709 / 1,754; VB 1,862 / 1,796 / 1,874; RATE 1,868 / 1,933 / 1,990; the engine 2,000. So no noise-floor cell moved between plain and twice-rounded arithmetic on types. Three cells first came out 3 short (t3411-irr-r4-fresh-2 Calc!A90, t3411-irr-r4-fresh-3 Calc!A91, t3406-irr-num-boundary P!A83): an `X87` operator raised ValueError on an infinite operand. That now ends the run as "step not finite", as plain doubles do, and all three are `#NUM!` as in Excel.
- Bits: the RATE transplant (hold + run on, cap 200) is bit-exact on 1,932 of 2,555 cells; the engine on main is bit-exact on 1,857. Train/test 1,376 / 556 against the engine's 1,338 / 519. No candidate is accepted: the fit has no correctly rounded reference to beat, so this is a measurement, not a selection.
- Event tracer: `Candidate.event` (score.py) and `events` in the JSON give every cell's terminal event and step per run, such as `converged@7` or `flat@3 then converged@5`. The events are converged, zero-step stop, flat, flat after a zero step, held on flat, iterate at d >= 1 or = 1, step not finite, NPV out of range, step cap, held at the cap, past the far-root bound, held iterate rejected, and no sign change. Every candidate also exposes `trace`: Excel's value is an iterate of the RATE transplant's trace on 1,368 of 1,876 number cells.
- Transforms: `fitlab/irr_transforms.py` provides `scale_pow2` (exact, refuses an inexact scale), `negate`, `pad_leading` and `pad_trailing`. Unit tests (tests/test_fitlab_irr_driver.py) show that x2^k for k in -40..40 keeps the secant's iterates bit-identical, that leading zeros are invisible, and that negation and trailing zeros keep the root.
- docs/architecture/fit-lab.md lists the fit and documents `event` and `typed`. No eval delta (tools only).
