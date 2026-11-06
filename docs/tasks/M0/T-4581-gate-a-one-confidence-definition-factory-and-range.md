---
id: T-4581
title: "Gate A: one CONFIDENCE definition factory and range check shared by CONFIDENCE.T and CONFIDENCE.NORM"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4101]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
CONFIDENCE.T, CONFIDENCE.NORM and CONFIDENCE share one definition factory, or at least one range check (directive 3).
Each keeps only its own quantile, its own evidence and its own domain rules.

## Context
- T-4102 built CONFIDENCE.T in defs/student_t.py on T.INV.2T, because T-4101's defs/confidence.py was not on main.
  T-4101 is PR #3595.
  - Its `confidence_margin` checks sd·t, sd/√n, t/√n and the margin against a normal double range, and raises
    NoValueError (unprobed) outside it.
  - Its `_confidence_t` treats a negative sd as NoValueError (unprobed) and sd 0 as a domain rule.
- T-4101's defs/confidence.py has a near-identical intermediate check (`RANGE`, the same four intermediates) and its
  own size, sd and alpha parameters.
- Once T-4101 is on main, keep one copy of the range check and the shared margin shape:
  - Move it to defs/_common.py, or make confidence.py's builder take the quantile.
  - Leave each function's evidence-specific rules where their sources are. Examples: CONFIDENCE.NORM's 1 - alpha/2
    rounding to 1, CONFIDENCE.T's alpha in (0, 1) and size cut at 1E10 + 1, and which names truncate the size.
- The committed tables must not change: `verify` keeps 0 differences for CONFIDENCE.T, CONFIDENCE.NORM and CONFIDENCE.

## Acceptance criteria
- [x] One range check (and, where the rules allow, one factory) serves the three CONFIDENCE definitions.
      `scripts/check_duplication.py` does not rise.
- [x] `rustsheet-accuracy verify` reports 0 differences for the three tables, and the tools tests pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- New evidence about either function's domain.

## Outcome
One margin and range check, `_common.confidence_margin(quantile, sd, n)` with `MARGIN_RANGE` = (smallest normal
double, 1E+300), now serves CONFIDENCE.NORM, CONFIDENCE (defs/confidence.py, quantile NORM.S.INV(1 - alpha/2)) and
CONFIDENCE.T (defs/student_t.py, quantile T.INV.2T(alpha, n - 1)). It checks the union of the two former intermediate
sets (sd, sd·q, sd/√n, q/√n, margin); no committed table point or eval sidecar is one where the added checks bite, as
`verify` and `make eval-math-check` show. `confidence.RANGE`, `student_t.CONFIDENCE_RANGE` and the old
`student_t.confidence_margin` are gone; `_confidence_t` narrows with `_common.scalars`. The definition factories stay
per file: the three share no parameter interval, source or domain rule, so each keeps its own evidence
(CONFIDENCE.NORM's 1 - alpha/2 rounding to 1, CONFIDENCE's size truncation, CONFIDENCE.T's sd 0 / negative sd and
size cut at 1E10 + 1). `verify` reports 0 differences for CONFIDENCE, CONFIDENCE.NORM and CONFIDENCE.T. No eval delta.

The bounds are now compared exactly on the mpf; the old confidence.py compared `abs(float(x))`, about one ulp looser.
Where the union can matter: for CONFIDENCE and CONFIDENCE.NORM the new z/√n check can never fail (z lies in about
1e-16..38). For CONFIDENCE.T the only new region is sd in (1e300, ~1e305) with alpha near 1, where no Excel cell exists
and the sampler stops at 1e100. A fidelity reviewer compared the old and new definitions on all 7921 candidate eval
cells: 0 differences. A unit test (`test_the_shared_margin_and_its_range_check`) covers an exact in-range value, an
overflowing and an underflowing intermediate.
