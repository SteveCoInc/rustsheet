---
id: T-3748
title: "Gate A defs share one scalar-argument and zero-pole helper module"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-3668, T-3670]
---

## Goal
The Gate A definitions in `tools/src/rustsheet_tools/accuracy/defs/` take their shared argument helpers from one
neutral module, not from another family's file, and no helper is defined twice.

## Context
- `scalars` lives in `defs/annuity.py` and is imported from there by discrete.py, inverse_trig.py (T-3669),
  trig.py (T-3668) and hyperbolic.py (T-3670): one family module serves as everybody's helper home.
- `_zero_pole` (a pole at 0, Excel's `#DIV/0!`) is defined identically in trig.py and hyperbolic.py. The two landed in
  sibling PRs, so `scripts/check_duplication.py` could not see the copy until both were on main.
- A signed log-uniform draw (`_signed` in inverse_trig.py, the sign coin-flip in rounding.py, cashflow.py,
  annuity.py and hyperbolic.py) is a candidate for `sampler.py` next to `log_uniform`.
- Found by the architecture and quality reviews of T-3670.

## Acceptance criteria
- [x] `scalars` and the zero-pole domain helper are defined once, in a module that is not a function family (for
      example `accuracy/model.py` or a `defs/_common.py`), and every definition imports them from there.
- [x] A shared signed log-uniform draw in `sampler.py` replaces the per-family copies, or the task's Outcome says why
      not.
- [x] Every committed table still verifies (`rustsheet-accuracy verify` for each function: 0 differences), so the draw
      change moves no committed point.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing any definition's math or any table.

## Outcome
`scalars` and one `zero_pole` (for COT, CSC, COTH, CSCH; it replaces the identical copies in trig.py and
hyperbolic.py) now live in `defs/_common.py`; every definition imports them from there, and annuity.py no longer
serves as a helper home. The underscore keeps `defs/__init__.py`'s discovery (T-3830) from importing it as a family;
the discovery test skips `_` modules too.

Signed draw not unified: the per-family `_signed` helpers are not copies. They take different parameters (a
`[lo, hi]` log-uniform range with a negative fraction in aggregate.py, a fixed magnitude in erf.py, exp_log.py,
hyperbolic.py and inverse_hyperbolic.py, `[lo, hi]` with a 0.5 coin in inverse_trig.py) and consume the RNG in
different orders. A shared draw would move committed points, which the task forbids ("moves no committed point"),
so each stays with its sampler. If a later task wants one, it must prove the RNG stream is identical per family.

Tables: no definition math or sampler changed, so no committed point moves; the Python suite (1406 tests) and
Gate A's CI check are the evidence. No eval delta.
