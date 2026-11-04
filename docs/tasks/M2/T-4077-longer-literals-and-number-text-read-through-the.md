---
id: T-4077
title: "Longer literals and number text read through the x87 multiply after the 15-digit cut"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-3932]
---

## Goal
Formula literals and number text of 7 to 15 significant digits (and with positive exponents, if probed to follow the model) read as Excel reads them, so the 15-digit T-3945 point and the cells below pass.

## Context
- numbers.md "Decimals reach a double through an x87 multiply by 10^-n": T-3932 implements the read for digits
  as written below 1,000,000, n = 1..15, no positive exponent (`parse_excel_number` in
  crates/rustsheet-engine/src/number_format.rs, `x87_decimal_to_f64` in decimal_round.rs). Longer digit strings
  keep the nearest double, but one probed 15-digit point follows the model: `=0.190581435720431` is
  0.19058143572043101 (T-3945).
- Probe first: 7- to 15-digit literals and VALUE text at model-off points, digits beyond 15 (how the 15-digit cut
  composes with the multiply: truncate, then m × 10^-n of the cut digits?), positive exponents, and decimal-point
  forms (`=0.00000491`, `=4.91E-06`) next to the integer-mantissa `mE-n` the t3658 cases use. Then widen the bound
  in `parse_excel_number`, and `literal_text` (stringify/literals.rs) so printed literals still read back.
- Owned cells (still failing after T-3932): probes/t3945-atan-near-midpoint S!B203:B204, S!C1, S!C3, S!C5,
  S!C7:C10, and probes-numeric/t3643-r4-log-modulus C111. T-3997 (M5) confirms S!C8 and S!C10 once this lands.

## Acceptance criteria
- [x] An oracle-filled probes-numeric case covers the forms above; numbers.md records the rule for each.
- [x] The owned cells pass, with no regression in the literal-parsing probes or the gen suites.
- [x] Decision 0188 R4 guards for any widened range (hold-out, fresh sweep, Gate A envelope) in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- The xlsx stored-number reader (numbers.md "Stored numbers in an xlsx are not always parsed to the nearest double").

## Outcome
- Probe probes-numeric/t4077-long-literal-x87 (516 cells, excel-graph 2026-11-04, seeded search of 4.8M (m, n)
  draws, 667 model-off points): 72 model-off points at 7 to 15 digits, each as a decimal-point literal, its VALUE, an
  mE-n literal and its VALUE; 9 controls; 24 rows written with a positive exponent; 24 rows of 18 written digits.
  T-3658's frozen model fits 516/516, the nearest double 37/516.
- Rule (numbers.md "Decimals reach a double through an x87 multiply by 10^-n"): literals and number text read their
  first 15 significant digits, later digits truncated, through the x87 multiply at n = fraction digits less exponent
  less the cut. Rounding the cut instead scores 8/24 of the 18-digit rows, and the nearest double of the full text
  scores 1/24.
- Engine: `parse_excel_number` reads its first 15 digits and any exponent through the new `x87_text_to_f64`
  (decimal_round.rs, digits below 10^15). ROUND and ROUNDDOWN keep `x87_decimal_to_f64`'s probed bound of 1,000,000,
  and both share the private `x87_multiply`. `literal_text` pads to 15 digits so printed literals still read back.
  The unit test `short_number_text_reads_through_the_x87_multiply` now asserts the probed long and positive-exponent
  reads.
- Decision 0188 R4:
  - The structural model is T-3658's, frozen before this probe, so every cell is out of sample. The cut rule
    (truncate) is numbers.md's "15-digit literal parsing" rule, not chosen by scoring.
  - Fresh sweep written after the code was frozen: probes-numeric/t4077-long-literal-x87-fresh (258 cells, unused
    hits, seed 4078) fits 258/258.
  - No constant is fitted, so there is no `FITTED` tag. No Gate A table cell is a literal read; `make check-fast`'s
    accuracy gate is unchanged.
- Owned cells: probes/t3945-atan-near-midpoint now passes 214/214, and probes-numeric/t3643-r4-log-modulus 540/540.
- Eval delta: probes known failures 4025 -> 4011 (14 fixed), 0 regressions; probes-numeric +2 cases, 774/774.
