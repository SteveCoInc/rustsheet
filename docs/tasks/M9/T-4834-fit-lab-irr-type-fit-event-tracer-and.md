---
id: T-4834
title: "Fit lab: irr-type fit, event tracer and path-invariant transforms for IRR"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] A fit `irr-type` in `fitlab/kernels/` over every IRR probe case (errors scored) with candidates for the base
      replica, the VB handlers and the RATE driver; its counts match feasibility.py's types within a few cells (the
      exact arithmetic may move noise-floor cells; the Outcome lists the moved ones).
- [ ] Candidates expose `trace` and the terminal event and step per cell, in the JSON.
- [ ] Transform helpers (x2^k scaling, negation, leading and trailing zero padding) with unit tests that x2^k keeps a
      scale-invariant replica's iterates bit-identical.
- [ ] docs/architecture/fit-lab.md's shipped-fit table lists the fit.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probe cases (T-4838).
