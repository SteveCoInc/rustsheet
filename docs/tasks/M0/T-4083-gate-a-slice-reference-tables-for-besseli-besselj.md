---
id: T-4083
title: "Gate A slice: reference tables for BESSELI, BESSELJ, BESSELK and BESSELY"
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
BESSELI, BESSELJ, BESSELK and BESSELY each get a cited mpmath Gate A definition and a committed, oracle-filled `s1`
table, and `make check` runs them. Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0270 (T-3800) classified these four as needing a table. They are
  the last missing tables of T-3800's group.
- The arguments are a real x and an integer order n, which the generator's `real` and `int` kinds drive.
- Domain and error regions to read from engineering.md:
  - "BESSELJ / BESSELY (T-0900)";
  - "BESSEL order cap": the 100,000 `MAX_ORDER` DEVIATION and Excel's own 10,000,000 cap. Keep the table's orders
    below 100,000, or draw the cap points as `domain` rules;
  - "BESSELK underflow, BESSELI overflow and BIT shifts of 49 to 53 (T-2421)", "BESSELK near the bottom of the double
    range (T-2550)", "BESSELI overflow point (T-2613)": BESSELI is `#NUM!` from |x| = 709.7, and BESSELK flushes to 0.
- Excel's BESSELJ is about 1e-7 from the math (decision 0188 R3), inside G = 1e-4, so Excel is the target. A point where
  Excel is more than G off is listed in the Outcome as an R3 candidate for an owner ruling, not marked.
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`). mpmath has
  `besseli`, `besselj`, `besselk` and `bessely`.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/` (a new `bessel.py`), each citing its
      sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [ ] A committed, oracle-filled `s1` table per function. `rustsheet-eval accuracy` reports `missing tables` lower by 4.
      Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The text-argument functions (T-4084, T-4085).
