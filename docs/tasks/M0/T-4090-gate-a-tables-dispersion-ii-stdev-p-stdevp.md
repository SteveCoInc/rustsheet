---
id: T-4090
title: "Gate A tables, dispersion II: STDEV.P, STDEVP, STDEVPA, VAR.P, VARP, VARPA and AVEDEV"
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
STDEV.P, STDEVP, STDEVPA, VAR.P, VARP, VARPA and AVEDEV each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (statistics.md): "VAR/STDEV switch between a one-pass and a two-pass formula", "VARA family: single-cell
  reference arguments". Share the dispersion definitions with T-4089 (one module, directive 3).
- AVEDEV is the mean absolute deviation from the mean.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- **Definitions:** `accuracy/defs/dispersion.py`, the module T-4089 started, now covers the dispersion II functions.
  Every function works over the literals' doubles with the exact mean.
  - VAR.P, VARP and VARPA are the exact population variance S / n. STDEV.P, STDEVP and STDEVPA are its square root.
  - AVEDEV is the exact mean absolute deviation.
  - The A-forms over numbers equal the plain forms, as in T-4089.
  - Domain: the population forms of one number are 0. They are `#NUM!` when the one-pass sum of squares overflows
    (statistics.md, "`VAR.P` of 1E+200 twice"; reuses `squares_overflow`).
  - AVEDEV is `#NUM!` when its sum of the numbers or its sum of absolute deviations overflows. This is inferred and
    not probed; the draws never reach it.
  - DEVSQ's overflow check and AVEDEV's share one helper, `_two_pass_overflow` (directive 3).
  - The draws are T-4089's. VAR.P and STDEV.P (`_xlfn`) take at most 254 numbers, per T-4089's quirk.
- **Tests:** tools/tests/test_accuracy_dispersion.py checks Microsoft's examples: VAR.P of the strength column is
  678.84, STDEV.P is 26.0545581, and AVEDEV(4,5,6,7,5,4,3) is 50/49. It also checks statistics.md's
  VAR.P({3,1,4,1,5,9}) = 269/36 and the domain rules.
- **Tables:** `eval/accuracy/tables/{STDEV.P,STDEVP,STDEVPA,VAR.P,VARP,VARPA,AVEDEV}/s1`, 120 points each, filled
  from excel-graph.
  - `rustsheet-eval accuracy` reports 0 failing, 0 table defects and 0 new known failures. Missing tables went from
    128 to 121.
  - R3 candidates (Excel more than 1E-4 off the math): none. The worst Excel-against-math gap is AVEDEV A61 at
    6.0E-8: offset data (-90629.958 three times), where Excel's double mean errs by about 1E-11 against deviations
    of about 5E-5. Per table, the worst gaps are 7.3E-12 (STDEV.P), 6.2E-16 (VAR.P), 3.2E-14 (VARP), 3.4E-15
    (VARPA), 2.3E-15 (STDEVP) and 1.1E-14 (STDEVPA).
- **Math sidecars:** registering AVEDEV gave probes/stat-order-floor-batch 7 new math cells (A256:A258, A269:A272).
  They were added to its sidecar with every existing entry kept as it was. `make eval-math-check` reports current.
- **Eval delta:** none, because no engine or eval-suite case changed.
