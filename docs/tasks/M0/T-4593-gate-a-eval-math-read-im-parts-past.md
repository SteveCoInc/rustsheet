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
- [ ] `make eval-math-check` is current (regenerate the sidecars if any cell gains a value); `make check` green;
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
- `number_literal` reads a literal whose decimal exponent is past ±400 whole: cutting it raised
  `decimal.InvalidOperation` (not ValueError) on a text such as `1e999999999+1i`, and the value is 0 or infinite
  either way (directive 5: no crash on untrusted text).
- Sidecars: regenerated for IMABS and IMARGUMENT (the only complex Gate A definitions) in probes and
  probes-numeric. One new file, probes/eng-impower-past-cut.math.json, with two cells (A15, A16); no existing
  sidecar changed. `make eval-math-check`: current.
- Oracle check of the new cells: 2 cells, 1 equal to Excel's double (A15
  `IMABS("0.7071067811865476+0.7071067811865476i")`, 0.9999999999999993), 1 far: A16 `IMABS(..)-1` is 0 in Excel
  against the math's -6.661338147750939e-16 (relative error 1). Excel's root `-` snaps this 3-ulp cancellation to
  0 (numbers.md "Near-zero snapping in addition and subtraction"); the composite math does not model the snap.
  Listed here only, no R3 mark (decision 0300).
- Last criterion: eval-math-check current and `make check-fast` green; reviewers not run in this session.
