---
id: T-4096
title: "Gate A tables, array regression: LINEST, LOGEST, TREND and GROWTH"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
LINEST, LOGEST, TREND and GROWTH each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- They return arrays, so each point reads one element through INDEX (for example `INDEX(LINEST(y,x),1,1)`); record
  the element in the point's formula. A multi-column x uses T-3686's 2-D literals.
- Sources (statistics.md): "LINEST and LOGEST: shapes, one point, collinear columns, exact fits and check order
  (T-1741)", "LINEST, LOGEST, GROWTH: probe findings", "LINEST, LOGEST and GROWTH in the gen-m9 sweep (T-2255)",
  "LINEST, LOGEST and GROWTH: cells left after T-1741 (T-1742)", "TREND: defaults, shapes, single points and error
  inputs", "LINEST and LOGEST report error values first (T-2455)".

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.
