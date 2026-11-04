---
id: T-4036
title: "Tidy the T-3680 accuracy budget code: split beta._betainc into a dispatcher, rename tables.time_limit to hang_guard, trim the budget comments"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3680]
---

## Goal
The Gate A accuracy generator's budget code reads plainly: one dispatcher for the incomplete beta's three regimes, a hang guard named as one, and comments that say each thing once. No sidecar byte changes.

## Context
Minor findings of the T-3680 design review (PR #3068), deferred so that PR could land:
- `tools/src/rustsheet_tools/accuracy/defs/beta.py` `_betainc`: the control flow (`first`, `raise first` inside a `try`,
  `raise first from None`) is hard to follow. Split out a `_via_series(a, b, x)` that maps NoConvergence to NoValueError,
  and a small dispatcher over the regimes (mpmath up to `MPMATH_SHAPE_CAP`, `_series` up to `SERIES_SHAPE_CAP`, mpmath
  past it), described once.
- `tools/src/rustsheet_tools/accuracy/tables.py`: the comment on `HANG_LIMIT` / `WORK_BUDGET` tells the calibration story
  twice; `time_limit` is now a hang guard (rename to `hang_guard`).
- `tools/src/rustsheet_tools/accuracy/model.py` `_WORK`: note that the budget state is per process and not thread-safe.

## Acceptance criteria
- [ ] `_betainc` is a dispatcher plus one series helper; behavior unchanged (`rustsheet-accuracy verify BETA.DIST` and
      `BETA.INV` show 0 differences; `rustsheet-accuracy math --check` passes for probes and probes-numeric).
- [ ] `tables.time_limit` is renamed `hang_guard`; the budget comment states the calibration once.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to which points compute or to a sidecar.
