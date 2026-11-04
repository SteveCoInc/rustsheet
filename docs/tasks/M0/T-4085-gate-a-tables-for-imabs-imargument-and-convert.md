---
id: T-4085
title: "Gate A tables for IMABS, IMARGUMENT and CONVERT, replacing their composite exemptions (after T-4084)"
milestone: M0
lane: harness
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4084]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
IMABS, IMARGUMENT and CONVERT get Gate A tables, and their `composite` lines leave `eval/accuracy/exempt.tsv`
(decision 0270).

## Context
- Decision 0270 (T-3800) made them `composite` until the generator has a text kind (T-4084).
- Kernels and fitted rules: engineering.md "IM* reciprocal kernels, IMABS and IMARGUMENT (T-2044)", "CONVERT constants
  and evaluation order", "CONVERT temperature", "CONVERT unit names". Open last-digit work: T-2538 (IMABS and
  IMARGUMENT), T-2046 and T-2047 (CONVERT).
- Removing an `exempt.tsv` line is a protected-path change: it needs a decision record.

## Acceptance criteria
- [ ] One cited mpmath definition per function, with unit tests; committed, oracle-filled `s1` tables.
- [ ] The three `exempt.tsv` lines are removed under a decision record. `missing tables` does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other IM* functions, which return complex text, not a number.
