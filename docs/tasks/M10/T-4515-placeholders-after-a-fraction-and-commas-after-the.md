---
id: T-4515
title: "Placeholders after a fraction and commas after the point: one model for the T-4507 probes"
milestone: M10
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One model for the digit placeholders after a fraction's denominator, and for commas after the decimal point, that
matches every probe of both, replacing the per-shape rules.

## Context
number-formats.md "Placeholders after a fraction" (probes/t4043-*, t3711-fixed-denominator-then-placeholder, from
T-4507) and "Commas after the decimal point" (t3598-decimal-part-commas rows 23, 24, 26, t3666-review-followups
A7:A12). The model in "Placeholders after a fraction" matched all its probes but not
probes/t4204-fraction-denominator-mixed; reconcile it with "A placeholder denominator prints its unused placeholders in
place". Subsumes T-4069. Hold-out: a fresh probe written after the model is frozen.

## Owned cases
Every visibly-off cell of these probes, or of the cells named (decision 0245):
- probes/t4043-fixed-denominator-cut
- probes/t4043-fixed-denominator-slots
- probes/t4043-frozen-model-holdout
- probes/t4043-placeholder-denominator-slots
- probes/t4043-separated-slots
- probes/t4043-trailing-shapes-holdout
- probes/t3711-fixed-denominator-then-placeholder
- probes/t3711-placeholders-after-fraction
- probes/t3598-decimal-part-commas
- probes/t3666-review-followups A7:A12 (commas; the rest of the case is T-4508's)

## Acceptance criteria
- [x] The t4043-* probes, t3711-fixed-denominator-then-placeholder and the comma cells above pass, with no regression.
- [x] A hold-out probe made after the model froze passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Locale tables (T-4508).

## Outcome
- One model for the placeholders after a fraction (number-formats.md "Placeholders after a fraction"):
  - A fixed denominator followed by placeholders is picked up to 9999999.
  - `picked_denominator` prints the code's digits and hands on spilled and lacking digits. A placeholder denominator
    hands on its lacking `?` slots.
  - `trailing_digits` prints each placeholder with the owed zeros and the space-due state.
  - A blank fraction blanks each placeholder to the width of what it would print.
  - It was first fitted in a Python sim over every cell of the owned probes plus t4204, then ported.
- Commas (parser.rs `Token::Comma`):
  - A `,` after a placeholder or after the point is inert when a placeholder follows anywhere later in the section;
    with none after it, it scales.
  - After a decimal-part literal, a `,` run prints one `,`.
  - After a fraction, a `,` between placeholders does not group.
- Every owned case passes (all t4043-*, t3711-*, t3598-decimal-part-commas, t3666-review-followups), and so do
  t4204-fraction-denominator-mixed and t4069-fixed-denominator-tail.
- Hold-out (structural model, no `FITTED` constant):
  - probes/t4515-fraction-tail-holdout: 147 cells, written after the first freeze. 140 passed at the freeze. Two comma
    refinements and a `?`-before-`.` refinement followed.
  - probes/t4515-fresh-sweep: 96 cells, written after that second freeze. It confirmed the comma refinements: rows 9-18
    pass except row 15, `0.,|0`, which is an older bug in placing a literal after the point (`0.|0` alike), now
    T-4595. It falsified the `.` refinement, which was reverted.
  - A tail with a `.` among its placeholders is a regime of its own, now T-4594: hold-out E6 and fresh-sweep rows 1-8.
    The hold-out passes everything else (146/147).
- Eval delta: probes known failures 6659 -> 5869 (820 fixed, 30 new-coverage failures in the two new cases, all owned by
  T-4594 and T-4595), 0 regressions in every suite. The visible-parity lock grows by those 30 new-case cells only.
  Roundtrip failures 3377 -> 3375.
- Follow-ups: T-4594 (a tail with a `.`, opus), T-4595 (a literal right after the point, sonnet).
