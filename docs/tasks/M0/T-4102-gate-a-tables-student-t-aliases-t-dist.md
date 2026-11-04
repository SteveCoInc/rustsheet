---
id: T-4102
title: "Gate A tables, Student t aliases: T.DIST.2T, T.DIST.RT, TDIST and CONFIDENCE.T"
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
T.DIST.2T, T.DIST.RT, TDIST and CONFIDENCE.T each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/student_t.py` (T-3358). TDIST takes `tails`. CONFIDENCE.T is T.INV.2T(alpha, n-1)·sd/sqrt(n).
- Sources (statistics.md): "T.DIST.RT and T.DIST.2T are not 1 - T.DIST", "T.DIST.RT follows the rounded y beyond t = 1
  (T-0969)", "T.DIST.RT is BETA.DIST's incomplete beta, switched at the mean (T-0969)", "T.DIST.RT's noise beyond t = 1
  grows with a ln y (T-0969)". T-3592 owns 1 CONFIDENCE.T cell inside Gate A.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.
