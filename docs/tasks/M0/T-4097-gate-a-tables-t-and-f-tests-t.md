---
id: T-4097
title: "Gate A tables, T and F tests: T.TEST, TTEST, F.TEST and FTEST"
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
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green
- [ ] reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/sample_tests.py`: one definition each for T.TEST, TTEST, F.TEST and FTEST.
  Each is the exact value on the literals' doubles: exact means and sums of squared deviations (dispersion's
  `squared_deviations`), the exact differences for a paired test, Welch's fractional df untruncated, then tails x the
  Student t tail (`student_t.lower_tail`) or 2 min of F.DIST's two tails (`fisher.lower_tail`, `fisher.upper_tail`).
  The draws reuse AGGREGATE's `array_of` and paired_regression's `offset_series`, `constant_series` and `constant_exact`
  (made public for this; no copy).
  - Domain errors, each from statistics.md: tails or type out of range `#NUM!` (before the size check); a paired test of
    different lengths `#N/A`; one pair, fewer than two numbers on a side (type 2, F.TEST) and zero variance (the
    differences; both samples for types 2 and 3; either sample for F.TEST) `#DIV/0!`; the evidenced
    `{1E+200,1E+200,-1E+200}` overflow `#NUM!` for type 2 and F.TEST only; and, from gen-m9 607e85cfee83a689 A132, a
    Welch test of one number beside a constant sample `#DIV/0!`.
  - No math value where nothing evidences Excel's result: a Welch test of one number beside a varying sample, any other
    overflow shape (paired, Welch, one form only), an inexact constant, a subnormal literal or an intermediate outside
    the normal range.
  - Before generating, the definitions were run over every one-call cell of these functions in eval/suites: 302 fit
    them, 242 numbers and 60 errors, with 0 mismatches either way (one Welch cell, A132 above, moved from no value to
    the domain rule it evidences).
  - Tests in tools/tests/test_accuracy_sample_tests.py: Microsoft's examples, the t1049-ttest and t1049-ftest Excel
    values, the evidenced domains, the unprobed points and the draws.
- Tables `eval/accuracy/tables/{T.TEST,TTEST}/s1` (200 points each) and `{F.TEST,FTEST}/s1` (120 each), 640 in all,
  filled from excel-graph with 0 failures. Excel gives an error on 57 points; the math gives a domain error with the
  same code on 56 of them.
- `rustsheet-eval accuracy`: 0 failing, 0 table defects, `missing tables` 58 to 54. 638 points inside the envelope.
  The engine is bit-exact with Excel on 214 of the 583 numeric points (T.TEST 54, TTEST 50, F.TEST 55, FTEST 55) and on
  every error point but one. Two known failures, both paired TTEST points, owned by the new task T-4579:
  - A136 is Excel `#DIV/0!` where the differences (18.375 and 18.374999999999996) are not constant. The one-pass
    variance of the differences is negative there, and only there; the hypothesis is recorded, not coded as a rule.
  - A101 (offset pairs, p 6.3E-70): Excel is 1E-15 from the math and ours 1.6E-9 off.
  Both are in statistics.md "T.TEST, TTEST, F.TEST and FTEST: Gate A tables (T-4097)".
- **Excel kept as the target (decision 0300):** no point is more than 1e-4 relative off the math, neither in the tables
  (the largest gap is T.TEST A65, 4.8e-11) nor in the 272 probes and probes-numeric cells with a math value. The list is
  empty, and the semantics entry says so.
- Math sidecars: `rustsheet-accuracy math --function` over the four functions for probes and probes-numeric. That gives
  10 new sidecars and 40 lines added to stat-argument-modes, and no existing cell changed. `make eval-math-check`
  prints "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
