---
id: T-4593
title: "Gate A eval math: read IM* parts past 15 digits by cutting them"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
An eval cell whose complex-number text has a part longer than 15 significant digits gets a math value. The part is read
as Excel reads it, cut to 15 digits.

## Context
- T-4084's `eval_math.fit_text` skips such cells ("complex number part past 15 significant digits").
- engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" (probes/eng-inumber-grammar-digits)
  shows Excel keeps 15 significant digits of each IM* part and cuts rather than rounds:
  `IMREAL("1.2345678901234567+2.2345678901234567i")` is 1.23456789012345.
- `fitlab.sweep.number_literal` already cuts a literal to 15 digits that way. Reuse it for each part, in `fit_text`
  and in `sampler.math_arg`, which today reads each part with `float()`.

## Acceptance criteria
- [x] A complex text with a part past 15 digits fits a text parameter, and the math receives the cut parts. A test
      uses the probe's cells.
- [x] Subnormal parts are still skipped unless evidence for them is added.
- [x] `make eval-math-check` is current (regenerate the sidecars if any cell gains a value); `make check` green;
      reviewers approve.

## Out of scope
- IM* grammar beyond what `parse_complex_text` reads.

## Outcome
- `fitlab.sweep.parse_complex_text` takes a `read` for each written part (default `float`, unchanged for
  scoring IM* results); the new `read_complex_arg` passes `number_literal`, so an IM* argument's part is cut (not
  rounded) to 15 significant digits. `eval_math.fit_text` no longer skips a part past 15 digits (its
  `_TEXT_NUMBER` check is gone) and checks the cut parts for subnormals; `sampler.math_arg` (and through it
  `composite`) gives the math the cut parts. Drawn Gate A texts are already 15-digit literals, so the tables do
  not move.
- Evidence: engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)",
  probes/eng-inumber-grammar-digits (excel-graph, 2026-11-02): `IMREAL("1.2345678901234567+2.2345678901234567i")`
  is 1.23456789012345 (a cut; rounding gives ...346). The new test reads all six texts of that probe and checks
  the cut parts against Excel's IMREAL and IMAGINARY (B1:C6). Subnormal parts are still skipped
  (`1.2345678901234567E-310+1i`); no evidence for them was added.
- `number_literal` reads a literal whose decimal exponent is past ±400 whole (0 or infinite either way; cutting
  it would leave Decimal's exponent range), and turns any `ArithmeticError` from Decimal (an exponent of 19 or
  more digits: `1e9999999999999999999`, `1e-9999999999999999999`, `0e9999999999999999999`) into `UnparsedError`,
  a ValueError, so `fit_text`, `math_arg` and the arithmetic reader skip such text instead of crashing
  (directive 5). Tests cover both exponent signs, a zero mantissa and both sides of the ±400 guard.
- Evidence scope: IM* cells show the cut for positive and negative plain-notation parts with an `i` suffix.
  Besides eng-inumber-grammar-digits, existing excel-graph cells show negative parts cut
  (probes/t3912-imcsch-band-edges A40 = A42, t3643-imsech-grid Q17 = P16); over the IM* hyperbolic and trig
  cells whose parts the cut changes, a correctly rounded reference matches Excel on 34 cells only when it reads
  the cut parts, and on none only when it reads the float parts (76 agree either way, 35 match neither, 83 have no
  text oracle). That is recorded in engineering.md. Exponent-form parts and `j` texts past 15 digits follow from
  the VALUE analogy (numbers.md "15-digit literal parsing"); no IM* cell shows them.
- The fit lab's `im_hyperbolic.correctly_rounded` reads its argument with `read_complex_arg` too (the 34-to-0
  count above), with a test on t3643-imsech-grid Q17.
- Sidecars: regenerated for IMABS and IMARGUMENT (the only complex Gate A definitions) in probes and
  probes-numeric. One new file, probes/eng-impower-past-cut.math.json, with two cells (A15, A16); no existing
  sidecar changed. `make eval-math-check`: current.
- Oracle check of the new cells: 2 cells. 1 equals Excel's double: A15
  `IMABS("0.7071067811865476+0.7071067811865476i")`, 0.9999999999999993. 1 is far: A16 `IMABS(..)-1` is 0 in
  Excel against the math's -6.661338147750939e-16 (relative error 1). The left addend 0.9999999999999993 is
  6 ulp below 1, and Excel's root `-` snaps that cancellation to 0 (numbers.md "Near-zero snapping in
  addition and subtraction"). Composite math does not model the operator snap (decision 0297), and Excel is
  kept as the target (decision 0300). Listed here only, with no R3 mark.
