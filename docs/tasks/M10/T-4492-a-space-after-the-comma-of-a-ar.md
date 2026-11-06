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
TEXT under a `[$-locale,N00]` tag prints what Excel prints for every shape of probes/t4492-locale-comma-tag-shapes: spaces around the suffix (`, 200`, `,  200`, `,200 `) and a hex LCID before the comma (`[$-40C,200]mmmm` is "March").

## Context
probes/t4492-locale-comma-tag-shapes (24 cells, excel-graph 2026-11-04; number-formats.md "The `locale,N00` form"). Main fails five:
- A16 `[$-ar-SA, 200]0`, A20 `,  200` and A22 `[$-, 200]0` are "٤٥٠٠١" (ours "45001");
- A21 `[$-ar-SA,200 ]0` is "肆伍零零壹" (ours "45001");
- A2 `[$-40C,200]mmmm` is "March" (ours "مارس").
Hypothesis to test first: the suffix is read like an extended-hex tag (formatter/locale_tag.rs `extended_hex`; a space counts 0x20 - 0x30 = -16), so " 200" wraps to byte 2 and "200 " to byte 0x1F; `,x` and `,200,1` then land on ASCII as observed. A hex LCID before the comma is not a locale (`[$-40C,1]mmmm` and `[$-40C,200]mmmm` are "March", A1, A2, A23) while a BCP-47 name is (`[$-fr-FR,200]mmmm` is "mars", A3). Probe a few more suffixes to confirm (`,1FF`, `,-200`, `,g00`) before implementing. Code: formatter/locale_tag.rs `numerals` and the tag resolution.
  Owns probes/t4492-locale-comma-tag-shapes.case.json S!A2, S!A16, S!A20, S!A21 and S!A22.

## Acceptance criteria
- [ ] probes/t4492-locale-comma-tag-shapes passes in full; the rule is in number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
