---
id: T-3711
title: "A lone 0 placeholder after a fraction prints its zero"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(12.5,"# ?/?|0 ""12""|\5")` prints "12 1/2|0 12|5" as Excel does: a `0` placeholder after a fraction prints its zero.

## Context
Found by T-3532: probes/t3532-numeral-digit-fields row 14, all columns (A14 "١٢ ١/٢|٠ 12|5"; ours "١٢ ١/٢| 12|5" drops the zero, also with ASCII digits). Probe the neighbouring forms (`# ?/? 0`, `# ?/? #`, `# ?/? 00`, a value whose fraction is zero) before generalizing. Semantics: number-formats.md "Numeral-system digits" (fields bullet) and "Fraction formats". Code: formatter/fraction.rs `format_fraction`.

## Acceptance criteria
- [x] probes/t3532-numeral-digit-fields row 14 passes (S!A14:G14, 7 cells); a probe covers the neighbouring forms.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done (general rule for placeholder denominators). Probed 20 forms over 6 values (probes/t3711-placeholders-after-fraction):
placeholders after the denominator print `0` (`0`, `#`) or a space (`?`), `.` prints, `,` vanishes, and a blank fraction
blanks through the last of them (`format_fraction`). number-formats.md "Placeholders after a fraction". Eval:
t3532-numeral-digit-fields S!A14:G14 pass, the new case 117/120 at the time; T-4043 (same branch) makes it pass in full. A second probe
(probes/t3711-fixed-denominator-then-placeholder, 88 cells) found that a fixed denominator followed by a placeholder
stops being fixed; filed as T-4043, which owns that case and A13, C13, E13 of the first.
