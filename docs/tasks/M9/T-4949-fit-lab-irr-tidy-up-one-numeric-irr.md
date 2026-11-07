---
id: T-4949
title: "Fit lab IRR tidy-up: one numeric-IRR cell predicate, a public running-product NPV, irr_secant_d as the driver's base case"
milestone: M9
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The fit lab's IRR kernels share one cell predicate, one public NPV and one secant loop, with every fit's scores unchanged.

## Context
MINOR findings of the design review on #3977 (T-4834, the `irr-type` fit):
- `irr_driver._numeric_irr` is `solvers._literal_irr` without the `"{" in cell.formula` test. Make it one helper in kernels/solvers.py with a flag for the array-literal condition, and have both predicates call it.
- `irr_driver` imports the private `irr_secant_d._npv_v_running`. Make it public (rename to `npv_v_running`) and update its callers.
- `irr_driver._secant` with `Options()` is the base case of `irr_secant_d.irr_secant_d`'s run, apart from its 50-step cap and stall rule. Either make `irr_secant_d` call the driver where they agree, or document in each why they stay separate.
- irr_transforms.py has no user yet besides the unit tests. Its docstring should name T-4838 as the consumer.
- Guard `max(abs(c) for c in flows)` in `_secant` with `default=0.0` (the #3977 security review; unreachable through `irr()` today).

## Acceptance criteria
- [ ] `uv run rustsheet-fitlab irr`, `irr-type` and `irr-fallback` give the same counts before and after (record them in the Outcome).
- [ ] tests/test_fitlab*.py pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New candidates or changed scoring.
