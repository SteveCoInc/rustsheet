---
id: T-4104
title: "Gate A tables, chi-squared, gamma and beta aliases: CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST and BETAINV"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3925]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST and BETAINV each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/gamma.py` and `defs/beta.py` (T-3529) and T-3925's GAMMA.DIST and CHISQ.DIST.RT definitions. CHIDIST is
  the upper tail.
- Sources (statistics.md): "Incomplete gamma sweep (T-1045)", "Chi-squared quantile at df 2 and gamma quantile at alpha 1
  are closed forms (T-2860)", "BETAINV is #NUM! where its quantile underflows, but 0.5 at a tiny alpha and a high p
  (T-2435)", "Legacy distribution names sweep (T-1047)", "Legacy distribution names sweep, second family (T-1048)".

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.
