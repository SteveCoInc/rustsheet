---
id: T-4389
title: "TEXT .000 fraction digits below 0.06 s: the offset past the 15-digit cut"
milestone: M10
lane: functions-text
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
TEXT's `.000` fraction digits match Excel where the remainder of a second is below about 0.06 s, which Excel rounds up
past what T-4370's 15-digit cut allows: `TEXT(0.0045/86400-36*2^-77,"[h]:mm:ss.000")` is "0:00:00.005", ours
"0:00:00.004".

## Context
T-4370 reads the fraction digits from the half-step cut's remainder `r` of a second rounded to 15 significant digits
(formatter/elapsed.rs `fraction_digits`; docs/excel-semantics/number-formats.md "Fraction digits of a second at a
tie"). Under `.000` with `r` below about 0.06 Excel behaves as if 1.6E-17 to 1.9E-17 s were added before that cut:
0.0045 flips between 2.12E-17 and 2.35E-17 s below the tie, 0.0075 between 1.91E-17 and 2.37E-17, 0.0015 and 0.0025
round up at every probed offset (down to 1.8E-17 below), 0.0545 at 5.9E-17 below. `.0` and `.00` show no offset
(0.005 under `s.00` flips 0.45 to 0.68 units of the 15th digit below). Neither a decimal grid (14 to 17 digits) nor a
binary snap of `r` fits all of it (docs/research/t4370/README.md). Probe the `.000` remainder between 1E-4 and 0.1 s
more densely, and check whether the offset is absolute in seconds or in serial days.

Visibly-off cells this task owns (all `.000` with a remainder below 0.06 s):
- probes/t4370-ulp-map A52, A86:A91
- probes/t4370-ulp-map-2 A1:A15, A18:A29, A47:A50, A52:A63, A76:A83, A243:A244, A313:A314

Also confirm on excel-desktop (M14, Phase 4, decision 0039, when it is available) the wrap points that rest on
excel-graph alone and on a single ulp: t4370-ulp-map-2 A417 ("41.0") and A418 ("42.0"), and t4333-count-ulp-scan
E6 ("3733.0").

## Acceptance criteria
- [x] The listed cells pass and no other cell of the t4323, t4332 and t4370 fraction cases moves.
- [x] A structural model: the R4 hold-out and a fresh sweep generated after it is frozen confirm it.
- [x] t4370-ulp-map-2 A417, A418 and t4333-count-ulp-scan E6 are checked on excel-desktop once it is available, or the
      semantics entry says they rest on excel-graph alone.
- [x] `make check` green; reviewers approve.

## Out of scope
- The `[s]`/`[h]`/`[m]` counts before the fraction (T-4333).

## Outcome
- Model: under `.000` only, Excel cuts the serial as if 2^-72 days (1.83E-17 s) were added after the half step.
  - Where it acts: the double addition can move the serial only below 2^-18 days. In [2^-19, 2^-18) the offset is
    half an ulp and rounds to even.
  - FITTED constant: every offset in [1.74E-17, 1.86E-17] s scores all 538 `.000` cells of the t4323/t4332/t4370
    fraction cases (T-4370's model 476), and 2^-72 days is the power of two inside that window. No operation that
    would produce it was found.
  - Code: `millisecond_offset` in formatter/elapsed.rs, tagged
    `FITTED(T-4389, 538 cells, offset 1.74E-17..1.86E-17 s, serial below 2^-18 days, .000 only)`.
- Seconds or days: the offset is in serial days. Both candidates were written down before the hold-out; the hold-out
  chose days (31 separating cells), and the confirmation batch drawn after review confirms it (40 of 40 new
  separating cells; seconds 0).
- R4 guards (decision 0188). Counts are bit-exact text; the 4-ulp and 15-digit columns do not apply to TEXT.
  - Train: 538/538 `.000` cells, 1664/1668 overall. The 4 misses are three `[s]` counts (T-4333) and t4370-ulp-map-2
    A418.
  - Hold-out: probes/t4389-holdout, 368/368 (T-4370 337, correctly rounded 81). It is a new probe, generated before
    scoring with the 10-point margin fixed, in place of a 30% split, because every existing `.000` cell was in the fit.
  - Fresh, after the freeze:
    - probes/t4389-fraction-fresh: 120/120 (42, 39).
    - probes/t4389-confirm: 69/80 (51, 31). The band cells A1:A40 score 29/40, against 11 for a gate at 2^-19 days
      (the first version of this PR, dropped after review). The 11 band misses are owned by follow-up T-4419.
  - Gate A: not applicable (TEXT output).
  - Reproducibility floor: the flips the fit rests on sit 36 to 40 ulp of the serial below the tie, above the 4-ulp
    floor.
- Excel-desktop: t4370-ulp-map-2 A417, A418 and t4333-count-ulp-scan E6 still rest on excel-graph alone; excel-desktop
  is not available (M14 Phase 4, decision 0039). number-formats.md says so.
- Unit test: `millisecond_ties_take_the_serial_offset` (formatter/test/test_time.rs). Research: docs/research/t4389.
- Eval delta: probes fixed 62 cells, every cell this task owned (t4370-ulp-map A52, A86:A91; -2 A1:A15, A18:A29,
  A47:A50, A52:A63, A76:A83, A243:A244, A313:A314). New cases: t4389-holdout and t4389-fraction-fresh pass in full;
  t4389-confirm passes 69 of 80, and its 11 misses are T-4419's. No regression.
- Follow-up: T-4419, the 11 band cells.
