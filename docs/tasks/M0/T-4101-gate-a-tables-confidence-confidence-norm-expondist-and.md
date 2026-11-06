---
id: T-4101
title: "Gate A tables, CONFIDENCE, CONFIDENCE.NORM, EXPONDIST and WEIBULL"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3783, T-3785]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
CONFIDENCE, CONFIDENCE.NORM, EXPONDIST and WEIBULL each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- CONFIDENCE and CONFIDENCE.NORM are NORM.S.INV(1 - alpha/2)·sd/sqrt(n): reuse T-3783's normal quantile. EXPONDIST and
  WEIBULL reuse T-3785's EXPON.DIST and WEIBULL.DIST definitions.
- Sources (statistics.md): "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)",
  "WEIBULL.DIST sweep (T-1046)", "Legacy distribution names sweep (T-1047)".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/confidence.py`: CONFIDENCE.NORM and CONFIDENCE from one factory, the margin
  z*sd/sqrt(n) with z from normal.py's `standard_quantile(1 - alpha/2)` (nothing copied). The size truncates for
  CONFIDENCE only (`CONFIDENCE(0.05,1,100.9)`, `(..,1.5)`, `(..,0.9)`, statistics.md T-1047); CONFIDENCE.NORM's
  truncation is unprobed, so its non-integer-size eval cells get no math value.
- `continuous.py`: EXPON.DIST and WEIBULL.DIST are now factories, and EXPONDIST and WEIBULL are registered from them
  (statistics.md T-1047 and T-1048: equal to the modern names on every paired cell).
- Error domain, from the evidence: alpha outside (0, 1), sd <= 0 and size < 1 are `#NUM!` (m9-legacy-confidence
  A20-A28). One domain rule beyond the intervals: alpha <= 2^-53 is `#NUM!`, because Excel forms 1 - alpha/2 in
  doubles, which rounds to 1 there (`CONFIDENCE(1E-16,..)`, `CONFIDENCE.NORM(1E-100,..)`). This is a new quirk,
  recorded in statistics.md "CONFIDENCE.NORM forms 1 - alpha/2 in doubles (T-4101)". Excel agrees on every error
  point drawn: CONFIDENCE 20 `#NUM!`, CONFIDENCE.NORM 22, EXPONDIST 7, WEIBULL 22 `#NUM!` and 8 `#DIV/0!`. Each is a
  domain error of the definition too.
- Left out of the CONFIDENCE tables (redrawn): the cancellation region, where rounding 1 - alpha/2 to a double can move
  the value by more than 1e-10 relative (alpha below about 4E-8 or above about 1 - 1.1E-6), as GAUSS's region is. Also
  points whose sd, sd*z, sd/sqrt(n) or value leaves the normal double range below 1E+300 (unprobed).
- Before generating, the definitions ran over every literal one-call cell of the four functions in eval/suites. None
  has a math domain error where Excel gives a number, and none has a math number where Excel gives an error.
- Tables `eval/accuracy/tables/{CONFIDENCE,CONFIDENCE.NORM,EXPONDIST,WEIBULL}/s1`, with 200, 200, 120 and 200 points,
  filled from excel-graph with 0 failures.
- `rustsheet-eval accuracy`: 25,362 points in 153 tables, 0 failing, 0 table defects, no new known failures; missing
  tables 102 -> 98. All 720 new points are inside the envelope. 715 are inside through both Excel's 15 digits and the
  math; 5 (3 CONFIDENCE, 2 CONFIDENCE.NORM) through the math only.
- Bit-exact with Excel: EXPONDIST 120/120, WEIBULL 200/200, CONFIDENCE 107/200, CONFIDENCE.NORM 104/200. The rest are a
  few ulp off, from the central NORM.S.INV (T-1746).
- Worst table gap between Excel and the math: CONFIDENCE 5.2E-11, CONFIDENCE.NORM 5.2E-11 (both near the cancellation
  cut), WEIBULL 8.6E-14, EXPONDIST 9.7E-15. No table point is more than 1e-4 off.
- **Excel kept as the target (decision 0300):** in the eval suites, these points are more than 1e-4 relative off the
  math (none of them is in the tables):
  - CONFIDENCE at alpha = 1E-15, 1.6E-3: gen-m9/8dd2312c96effb1b A251 and c6cfda6bd10cae93 A73.
  - CONFIDENCE at alpha = 4.14159E-15, 3.0E-4: gen-m9/9cca17438a7e6f8a A229.
  - CONFIDENCE and CONFIDENCE.NORM at alpha = 1 - 1E-16, which are 0 against a positive math value:
    gen-m9/05d44f6dd6cfc567 A230, 8d3588d8df0812df A124, 8dd2312c96effb1b A346 and 146b46e74f2dac3b A349.
  - WEIBULL's density at x = 0 with alpha = 1 (probes-numeric/t1048-weibull A16), already under decision 0300 (T-4318).
  EXPONDIST and WEIBULL cells whose math is below 1e-300, such as `EXPONDIST(800,1,FALSE)` = 0, are inside decision
  0188 R2's absolute floor.
- Sidecars: `rustsheet-accuracy math --function` for the four functions on probes and probes-numeric. The change is
  additive: 6 sidecars gain cells, and m9-legacy-confidence gets a new one. `make eval-math-check` reports current.
- Unit tests: tools/tests/test_accuracy_confidence.py covers Microsoft's example, the T-1047 values, the error domain,
  the 2^-53 cut, truncation, and the draws avoiding the cancellation region. tools/tests/test_accuracy_continuous.py
  checks that the legacy names share the modern definitions, plus the legacy cells' values.
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
