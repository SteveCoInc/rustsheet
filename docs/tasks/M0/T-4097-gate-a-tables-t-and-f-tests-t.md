---
id: T-4097
title: "Gate A tables, T and F tests: T.TEST, TTEST, F.TEST and FTEST"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
T.TEST, TTEST, F.TEST and FTEST each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0271 (T-3799) classified these functions as needing a table.
- The generator's one-row array literals (T-3586) and 2-D literals (T-3686) drive the data arguments. Variadic data
  follows T-3549's SUM and AVERAGE definitions (the draw of a variable-length array and the near-zero snap rule):
  reuse it, do not copy it (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Sources (statistics.md): "T.TEST rounds y = df/(df+t^2) to a double before the tail (T-0969)", "T.TEST: an error
  sample first, then tails and type errors, then their range check (T-2256)", "F.TEST / FTEST: larger variance on top,
  Excel's rounded powers, an accurate remainder (T-1927, T-2361)", "F.TEST of a single value (T-2455)", "F.TEST: an error
  argument is that error; overflowing squares are #NUM!, KURT's #DIV/0! (T-2255)".
- Reuse the Student t and F distribution definitions (`defs/student_t.py`, the F definitions of T-3358).

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.
