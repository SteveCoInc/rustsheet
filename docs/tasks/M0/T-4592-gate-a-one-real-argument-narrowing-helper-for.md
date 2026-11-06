---
id: T-4592
title: "Gate A: one real-argument narrowing helper for the definitions"
milestone: M0
lane: harness
status: todo
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

## Acceptance criteria
- [ ] One helper or constant in `accuracy/model.py`, used by every definition that narrows an argument to a real;
      no `isinstance(..., (bool, tuple))` or `(bool, str, tuple)` tuple is left in `accuracy/defs`.
- [ ] No behaviour change: every committed table verifies (`rustsheet-accuracy verify`), `make eval-math-check` is
      current, and mypy is clean.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what any definition computes.
