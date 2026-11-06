---
id: T-4069
title: "Fixed-denominator fraction with a placeholder after it: '# ?/4|0' on 12.5 is '12 1/4|0', not '12 2/4|0'"
milestone: M10
lane: functions-text
status: in-progress
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
- [ ] probes/t3711-placeholder-after-fraction S!A13 passes; a probe covers the forms above; number-formats.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
