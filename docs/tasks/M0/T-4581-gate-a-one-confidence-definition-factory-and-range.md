---
id: T-4581
title: "Gate A: one CONFIDENCE definition factory and range check shared by CONFIDENCE.T and CONFIDENCE.NORM"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4101]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
CONFIDENCE.T, CONFIDENCE.NORM and CONFIDENCE share one definition factory, or at least one range check (directive 3).
Each keeps only its own quantile, its own evidence and its own domain rules.

## Context
- T-4102 built CONFIDENCE.T in defs/student_t.py on T.INV.2T, because T-4101's defs/confidence.py was not on main.
  T-4101 is PR #3595.
  - Its `confidence_margin` checks sd·t, sd/√n, t/√n and the margin against a normal double range, and raises
    NoValueError (unprobed) outside it.
  - Its `_confidence_t` treats a negative sd as NoValueError (unprobed) and sd 0 as a domain rule.
- T-4101's defs/confidence.py has a near-identical intermediate check (`RANGE`, the same four intermediates) and its
  own size, sd and alpha parameters.
- Once T-4101 is on main, keep one copy of the range check and the shared margin shape:
  - Move it to defs/_common.py, or make confidence.py's builder take the quantile.
  - Leave each function's evidence-specific rules where their sources are. Examples: CONFIDENCE.NORM's 1 - alpha/2
    rounding to 1, CONFIDENCE.T's alpha in (0, 1) and size cut at 1E10 + 1, and which names truncate the size.
- The committed tables must not change: `verify` keeps 0 differences for CONFIDENCE.T, CONFIDENCE.NORM and CONFIDENCE.

## Acceptance criteria
- [ ] One range check (and, where the rules allow, one factory) serves the three CONFIDENCE definitions.
      `scripts/check_duplication.py` does not rise.
- [ ] `rustsheet-accuracy verify` reports 0 differences for the three tables, and the tools tests pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New evidence about either function's domain.
