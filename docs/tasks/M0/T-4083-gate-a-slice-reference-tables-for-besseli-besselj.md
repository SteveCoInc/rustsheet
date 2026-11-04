---
id: T-4083
title: "Gate A slice: reference tables for BESSELI, BESSELJ, BESSELK and BESSELY"
milestone: M0
lane: harness
status: done
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
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/` (a new `bessel.py`), each citing its
      sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function. `rustsheet-eval accuracy` reports `missing tables` lower by 4.
      Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The text-argument functions (T-4084, T-4085).

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/bessel.py`: one definition each for BESSELI, BESSELJ, BESSELK and BESSELY,
  mpmath's Bessel functions of the truncated order.
  - Domain errors (Excel's `#NUM!`): a negative order; BESSELY and BESSELK of x <= 0; BESSELI past |x| = 709.7
    (T-2613); BESSELJ and BESSELY past the phase cut, |x| > 134217730 (T-2547).
  - Draws stay at orders <= 100,000, the engine's order cap DEVIATION, so the table never scores it. A tiny x is drawn
    with small orders only (the Miller overflow of `BESSELJ(1E-300,100001)`).
  - Tests in tools/tests/test_accuracy_bessel.py (A&S values, the semantics entries' Excel values, domains, draws).
- Tables `eval/accuracy/tables/{BESSELI,BESSELJ,BESSELK,BESSELY}/s1`, 120, 120, 120 and 119 points (one BESSELY point
  dropped: mpmath did not converge at x = 29487.9, n = 74119), filled from excel-graph with 0 failures.
- `rustsheet-eval accuracy`: 479 new points, all inside the envelope (196 on both sides, 283 through Excel's 15 digits
  only). The engine is bit-exact with Excel on all 479. 0 failing, 0 known failures added. `missing tables` 157 to 153.
- **R3 candidates, for an owner ruling (not marked; T-4196):** 8 points are more than 1e-4 relative off the math.
  Seven are BESSELI at |x| >= 250 with 2 <= n <= 11, up to 53% (`BESSELI(709,3)` 5.748E+305 against 1.224E+306). One is
  `BESSELJ(11.0627245753426,4)`, 0.14% next to a root. Recorded in engineering.md, "BESSELI at a large x and an order
  of 2 or more is far from the math (T-4083)".
- Not done here: regenerating the probes-numeric math sidecars so the Bessel eval cells get math values
  (`rustsheet-accuracy math --suite probes-numeric`). No acceptance criterion asks for it.
- Eval delta: none in the gated suites (tools, tables and docs only).
