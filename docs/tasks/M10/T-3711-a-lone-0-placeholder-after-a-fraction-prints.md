---
id: T-3711
title: "A lone 0 placeholder after a fraction prints its zero"
milestone: M10
lane: functions-text
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(12.5,"# ?/?|0 ""12""|\5")` prints "12 1/2|0 12|5" as Excel does: a `0` placeholder after a fraction prints its zero.

## Context
Found by T-3532: probes/t3532-numeral-digit-fields row 14, all columns (A14 "١٢ ١/٢|٠ 12|5"; ours "١٢ ١/٢| 12|5" drops the zero, also with ASCII digits). Probe the neighbouring forms (`# ?/? 0`, `# ?/? #`, `# ?/? 00`, a value whose fraction is zero) before generalizing. Semantics: number-formats.md "Numeral-system digits" (fields bullet) and "Fraction formats". Code: formatter/fraction.rs `format_fraction`.

## Acceptance criteria
- [ ] probes/t3532-numeral-digit-fields row 14 passes (S!A14:G14, 7 cells); a probe covers the neighbouring forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A digit placeholder after a fraction's denominator (separated by a literal) now prints a zero, or a space under `?`, and a
blank fraction blanks everything up to the last of them (formatter/fraction.rs). probes/t3532-numeral-digit-fields S!A14:G14
pass (7 cells). New probe probes/t3711-fraction-trailing-placeholders (excel-graph) covers `0`, `#`, `?`, `00`, `0#`, `0.0`
and the task's format at 12.5, 12 and 0.5: 34 of its 57 cells pass. The other 23 cells put a different placeholder
kind right after the `/` (`?/?0`, `?/#0`, `?/0?`, ...), a separate denominator-run rule, owned by T-4204 and recorded in
number-formats.md "Fraction formats". Eval delta: see the PR.
