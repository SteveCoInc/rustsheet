---
id: T-4106
title: "Gate A tables, GAMMA, GAMMALN, GAMMALN.PRECISE, PERMUT and PERMUTATIONA"
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
GAMMA, GAMMALN, GAMMALN.PRECISE, PERMUT and PERMUTATIONA each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0272 (T-3798) classified these functions as needing a table.
- A legacy name gets its own table that reuses the modern function's Definition (the default of T-3798: TINV already has
  its own table beside T.INV.2T). Where the alias's arguments differ (tails, a right tail, a cdf only), the shared
  Definition takes that into account rather than being copied (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- GAMMA and GAMMALN are Excel's own x87 routines, and Excel's value is a valid target within 15 digits (statistics.md
  "GAMMA is Excel's own x87 routine (T-1941, T-2129)", "GAMMALN is Excel's own x87 routine (T-1941, T-2128)", "GAMMA and
  GAMMALN sweep (T-1042)", "GAMMA is exact at small positive integers but not always correctly rounded").
  GAMMALN.PRECISE shares GAMMALN's kernel.
- PERMUT and PERMUTATIONA have integer results with a floating-point stage, so they need a table, as FACT and COMBIN did
  in decision 0228 (statistics.md "Overflowing arguments, PERMUT's limit and argument error order (T-1290)"). Reuse
  `defs/combinatorics.py`.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.
