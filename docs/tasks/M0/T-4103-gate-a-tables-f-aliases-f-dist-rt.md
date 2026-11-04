---
id: T-4103
title: "Gate A tables, F aliases: F.DIST.RT, F.INV.RT, FDIST and FINV"
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
F.DIST.RT, F.INV.RT, FDIST and FINV each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/fisher.py` (T-3358). FDIST and FINV are right-tailed, like F.DIST.RT and F.INV.RT.
- Sources (statistics.md): "F density is not the beta density, R's df or a GAMMALN formula (T-2513)", "Incomplete beta is
  TOMS 708 `bratio`: one kernel behind BETA.DIST, F, T, BINOM and NEGBINOM (T-2464)", "F.INV.RT saturates at
  3.696066238512854E+150/df1 in the far tail (T-2435)". T-3592 owns 36 F.DIST.RT cells inside Gate A.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.
