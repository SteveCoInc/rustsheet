---
id: T-4043
title: "A fixed denominator followed by placeholders picks the fraction and spills its digits"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`TEXT(0.0625,"# ?/4|0")` is " 1/4|60" as in Excel: a fixed denominator followed by digit placeholders no longer
fixes the denominator.

## Context
Found by T-3711 (probes/t3711-fixed-denominator-then-placeholder, all cells; probes/t3711-placeholders-after-fraction
A13, C13, E13). With a placeholder anywhere after a fixed denominator, Excel picks the fraction (0.0625 is 1/16,
0.3 3/10, 0.14159 14159/100000), prints the written denominator digit in place of the picked denominator's first
digit, and the picked denominator's other digits spill into the placeholders after it: under `0` or `#` they print
followed by the placeholder's own zero (`# ?/4|0` on 0.3 is " 3/4|00", on 0.123 " 123/4|0000", on 0.5 " 1/4|0"),
under `?` only one of them prints, or a space when there is none (`# ?/4|?` on 0.0625 is " 1/4|6", on 0.123
" 123/4|0", on 0.5 " 1/4| "). A literal after the denominator alone changes nothing (`# ?/4"x"` on 0.5 is " 2/4x").
Two-digit and longer fixed denominators: `# ?/16|0` on 0.5 is " 1/1 |0", `# ?/100|0` on 0.3 " 3/10 |0".
Semantics: number-formats.md "Placeholders after a fraction". Code: formatter/fraction.rs `format_fraction`
(the fixed-past-32768 path, `Denominator::Fixed` with `fixed == None`, is the closest model).

## Acceptance criteria
- [ ] probes/t3711-fixed-denominator-then-placeholder and probes/t3711-placeholders-after-fraction A13, C13, E13
      pass; a probe settles the largest denominator Excel picks here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Placeholders after a placeholder denominator (done by T-3711).
