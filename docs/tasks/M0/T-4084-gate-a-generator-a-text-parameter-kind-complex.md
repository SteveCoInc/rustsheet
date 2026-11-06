---
id: T-4084
title: "Gate A generator: a text parameter kind (complex numbers and CONVERT units)"
milestone: M0
lane: harness
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
The Gate A generator has a `text` parameter kind, so a definition can draw complex-number text and fixed-list text
(CONVERT's unit names) and the table and math sidecar carry it.

## Context
- Decision 0270 (T-3800) made IMABS, IMARGUMENT and CONVERT `composite`, although they have floating-point
  kernels, only because their arguments are text and `accuracy/model.py` `Kind` has `real`, `int`, `bool` and `array`.
- Decision 0228 did the same for the matrix functions until T-3686 added 2-D literals. This task is the counterpart.
- Shapes needed: a complex number `"a+bi"` / `"a-bj"` built from two drawn reals (the IM* parse rules are in
  engineering.md, "Leading plus in IM*..."), and a choice from a fixed list (CONVERT's units, engineering.md "CONVERT
  unit names").
- The text must reach the formula as a quoted string literal, and the math side must receive the parsed values.

## Acceptance criteria
- [ ] `Kind` gains a text kind with a complex-number sampler and a fixed-list sampler, with tests. The formula writer
      quotes it and escapes `"`.
- [ ] Existing tables regenerate byte for byte (no table changes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tables themselves (T-4085).
