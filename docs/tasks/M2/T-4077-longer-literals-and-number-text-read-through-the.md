---
id: T-4077
title: "Longer literals and number text read through the x87 multiply after the 15-digit cut"
milestone: M2
lane: engine
status: todo
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
- [ ] An oracle-filled probes-numeric case covers the forms above; numbers.md records the rule for each.
- [ ] The owned cells pass, with no regression in the literal-parsing probes or the gen suites.
- [ ] Decision 0188 R4 guards for any widened range (hold-out, fresh sweep, Gate A envelope) in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The xlsx stored-number reader (numbers.md "Stored numbers in an xlsx are not always parsed to the nearest double").
