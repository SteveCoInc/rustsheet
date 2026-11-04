---
id: T-4043
title: "A fixed denominator followed by placeholders picks the fraction and spills its digits"
milestone: M10
lane: functions-text
status: done
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
- [x] probes/t3711-fixed-denominator-then-placeholder and probes/t3711-placeholders-after-fraction A13, C13, E13
      pass; a probe settles the largest denominator Excel picks here.
- [x] `make check` green; reviewers approve (with T-3666 and T-4061 on one branch).

## Out of scope
- Placeholders after a placeholder denominator (done by T-3711).

## Outcome
Done. A fixed denominator is used as written only up to 32768 and with no digit placeholder after it; otherwise the
fraction is picked as past 32768 (last convergent, denominator up to 9999999: PI() gives 3 + 244252/1725033,
0.123456789 10/81), one path (`Choice`, formatter/fraction.rs). The picked denominator's digits past the code's print
after them under a `#`/`0` numerator and spill into the placeholders after it under a `?` one (`trailing_fill`); the
same slot rule covers placeholder denominators, which a new probe settled (probes/t4043-placeholder-denominator-slots,
20 cells: `# ?/?|0?` on 0.5 is " 1/2|00"). Evidence: probes/t4043-fixed-denominator-slots (285 cells),
t4043-fixed-denominator-cut (88), t3711-fixed-denominator-then-placeholder (88) and t3711-placeholders-after-fraction
(120) all pass; probes known failures 4427 -> 4350 (77 fixed), no regressions. Done alongside a refactor of the
format parser and fraction code: date tokens map through one helper and a `MinuteSlot` state decides minute or month
(formatter/parser.rs); a pure `comma_role` table decides what a `,` does (formatter/comma.rs); `format_fraction` is
split into choosing and rendering by `Zone`. Semantics: number-formats.md "Placeholders after a fraction" and
"Fraction formats".

Revised after review: the first `trailing_fill` was fitted to the probed shapes and failed 19 of 84 cells of a hold-out
(probes/t4043-trailing-shapes-holdout). probes/t4043-separated-slots (432 cells: every placeholder of 24 shapes in a slot
of its own) showed the mechanism: spilled digits, lacking digits, owed zeros and a due space, walked left to right
(number-formats.md "Placeholders after a fraction"); a blank fraction is the would-be text as spaces. The rule matches
every cell of all seven probes, including the hold-out and the 34 cells with no `?` slot. Since that hold-out
shaped the revision, a second one was written after the model was frozen (probes/t4043-frozen-model-holdout: 12 new
shapes, denominators 2-9999, x 9 values): 108/108 match with no change.
