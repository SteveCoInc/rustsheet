---
id: T-4492
title: "A space after the comma of a [ar-SA, 200] tag still picks the numeral system"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(45000.75,"[$-ar-SA, 200]0")` is "٤٥٠٠١" as in Excel: a space after the comma does not stop the numeral system.

## Context
Found by a T-3891 probe in a parallel session: probes/t3891-script-digits-typed-shapes Sheet1!F16 (ours "45001"). The other comma-tag shapes in column F pass on main: `,2`/`,20` give ASCII, `,201` Arabic-Indic, `,2000` full-width, and `,x` and `,200,1` ASCII. Probe a few more spacing forms first: `[$-ar-SA ,200]0`, `[$-ar-SA,  200]0`, `[$-ar-SA,200 ]0`, `[$-, 200]0`. Code: formatter/locale_tag.rs (the comma part of a tag, `numerals`). Semantics: number-formats.md numeral-system entries and coercion.md "Script digits in number text".
  Owns probes/t3891-script-digits-typed-shapes.case.json Sheet1!F16.

## Acceptance criteria
- [ ] probes/t3891-script-digits-typed-shapes Sheet1!F16 passes; a probe covers the spacing forms; the rule is in number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
