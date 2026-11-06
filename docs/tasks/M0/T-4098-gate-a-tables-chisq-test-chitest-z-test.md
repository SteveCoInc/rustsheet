---
id: T-4098
title: "Gate A tables, CHISQ.TEST, CHITEST, Z.TEST, ZTEST and PROB"
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
CHISQ.TEST, CHITEST, Z.TEST, ZTEST and PROB each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (statistics.md): "Z.TEST: an error array first; one data value is #DIV/0!; a lone non-number cell is #N/A
  (T-2256)", "PROB: an error data argument first, one value on each side is #DIV/0!, unequal counts #N/A (T-2256)",
  "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments".
- Reuse the chi-squared definitions (`defs/gamma.py`, T-3529) and the normal CDF (`defs/erf.py`). CHISQ.TEST takes two
  2-D arrays of the same shape (T-3686).

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.
