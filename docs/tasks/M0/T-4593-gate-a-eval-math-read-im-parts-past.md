---
id: T-4593
title: "Gate A eval math: read IM* parts past 15 digits by cutting them"
milestone: M0
lane: harness
status: todo
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
- [ ] A complex text with a part past 15 digits fits a text parameter, and the math receives the cut parts. A test
      uses the probe's cells.
- [ ] Subnormal parts are still skipped unless evidence for them is added.
- [ ] `make eval-math-check` is current (regenerate the sidecars if any cell gains a value); `make check` green;
      reviewers approve.

## Out of scope
- IM* grammar beyond what `parse_complex_text` reads.
