---
id: T-4592
title: "Gate A: one real-argument narrowing helper for the definitions"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The Gate A definitions narrow a `MathArg` to a real through one shared helper (`real_arg(x)`) or one `NonReal` constant
in `accuracy/model.py`, instead of each repeating an `isinstance` tuple.

## Context
- T-4084 added `str` and `mpc` to `MathArg`. That made mypy flag the definitions that narrow with
  `isinstance(x, (bool, tuple))`, so six defs files now spell `(bool, str, tuple)`. About 40 sites read that way and 11
  still read `(bool, tuple)` (`rg -n "\(bool, (str, )?tuple\)" tools/src/rustsheet_tools/accuracy`).
- The next `MathArg` member would mean editing every site again. Raised by T-4084's design review.
- T-4395 added an asserting `_common.scalar(a)`; `_common.scalars` still narrows with `(bool, tuple)`. Fold both into
  `real_arg`. The domain functions also need a predicate form (for example `is_real`), since `scalar` asserts
  (gamma_dist.py `_gamma_dist_domain` and the CHISQ domains). Raised by T-4395's design review.
- `sys.float_info.min` is still spelled locally in seriessum.py (`MIN_NORMAL`), aggregate.py, confidence.py, normal.py,
  paired_regression.py, percentof.py, product.py and sumx.py; move them to `_common.NORMAL_MIN` in the same pass.

## Acceptance criteria
- [x] One helper or constant in `accuracy/model.py`, used by every definition that narrows an argument to a real;
      no `isinstance(..., (bool, tuple))` or `(bool, str, tuple)` tuple is left in `accuracy/defs`.
- [x] No behaviour change: every committed table verifies (`rustsheet-accuracy verify`), `make eval-math-check` is
      current, and mypy is clean.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what any definition computes.

## Outcome
- `accuracy/model.py` holds the one `NON_REAL` tuple (`bool`, `str`, `tuple`, `mpc`): `Definition.check_domain` and
  the new `defs/_common` helpers narrow with it, so the next `MathArg` member is added in one place.
- `_common.scalar`/`scalars` are folded into `real_arg`/`real_args` (asserting), plus the predicate `is_real`
  (`TypeIs[mpf]`, so mypy narrows both branches) for the domains and tail maps (gamma_dist, normal, beta, student_t,
  fisher, matrix's INDEX, TRIMMEAN's percent, the order statistics' k). No `(bool, tuple)`, `(bool, str, tuple)`,
  `bool | tuple` or bare `isinstance(x, bool)` narrowing is left in `accuracy/defs` (fisher.py, already on main,
  included). The helpers live in `_common` beside the other shared definition helpers; the defs that imported
  `scalars` from it keep one import line, which also keeps the duplication ratchet at main's count (2791).
- The local `sys.float_info.min` / `2.0**-1022` spellings (seriessum `MIN_NORMAL`, remainder and cashflow_values
  `SMALLEST_NORMAL`, aggregate, confidence, normal, percentof, product, sumx, significance, exp_log's `LN_MIN`) now
  read `_common.NORMAL_MIN`; significance's private `_subnormal` was a copy of `_common.below_normal` and is gone.
  (paired_regression.py had no local spelling left on main.)
- No behaviour change: every registered definition verifies with 0 differences (`rustsheet-accuracy verify`, one call
  per function), `make eval-math-check` is current, `eval/suites` is untouched, mypy and pytest are clean.
- Eval delta: none.
- Left as is: `accuracy/sampler.py` narrows a drawn `Arg` (the literal, not a `MathArg`) with `(bool, str, tuple)`,
  and `composite.py`/`eval_math.py` keep their own `_SMALLEST_NORMAL`; both are outside `accuracy/defs` and this task.
