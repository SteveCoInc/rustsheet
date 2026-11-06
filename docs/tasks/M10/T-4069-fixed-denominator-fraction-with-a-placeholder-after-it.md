---
id: T-4069
title: "Fixed-denominator fraction with a placeholder after it: '# ?/4|0' on 12.5 is '12 1/4|0', not '12 2/4|0'"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(12.5,"# ?/4|0")` is "12 1/4|0" as in Excel.

## Context
Found by T-3711: probes/t3711-placeholder-after-fraction S!A13. With a fixed denominator and a digit placeholder after it, Excel prints numerator 1 where `# ?/4` alone prints 2 ("12 2/4"). The trailing `0` might change the denominator Excel uses (read as 40, or as 4 with a scaling) or the numerator rounding. Probe first: `# ?/4|0` on 12.25, 12.5, 12.75, 12.1; `# ?/4 0`, `# ?/4|#`, `# ?/4|00`, `# ?/8|0`, `# ?/10|0`, `# ?/4"x"0`. Code: formatter/fraction.rs `format_fraction`, formatter/parser.rs (fixed denominator digits).
  Owns probes/t3711-placeholder-after-fraction.case.json S!A13.

## Acceptance criteria
- [x] probes/t3711-placeholder-after-fraction S!A13 passes; a probe covers the forms above; number-formats.md records the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
-

Note (T-4507): T-4515 takes this case into one model of the placeholders after a fraction.

## Outcome
Probed the shapes: probes/t4069-fixed-denominator-tail (26 cells, excel-graph 2026-11-06). A digit
placeholder after a fixed denominator makes Excel pick the fraction as for placeholders as wide as
the denominator and print the code's digits cut to the picked denominator's length, as a fixed
denominator past 32768 does (formatter/fraction.rs `format_fraction`, `tail_placeholders`).
probes/t3711-placeholder-after-fraction A13 passes; the new case passes 25/26. Its A4 (`# ?/4|0`
on 12.1 is "12 1/4|00") is filed as T-4526. number-formats.md "After a fixed denominator";
0 regressions. After merging T-4507's probes (#3527) the same rule also passes 127 of their cells
that T-4515 owns (t4043-fixed-denominator-slots 45, t4043-separated-slots 40,
t3711-fixed-denominator-then-placeholder 17, t4043-fixed-denominator-cut 15, and 10 more); the rest
stay with T-4515's single model.
