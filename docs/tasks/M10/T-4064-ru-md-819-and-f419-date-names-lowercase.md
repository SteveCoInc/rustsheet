---
id: T-4064
title: "ru-MD [819] and [F419] date names: lowercase month (and weekday under 819) where ru-RU capitalises"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(45000.75,"[$-819]mmmm|ddd|...")` is "март|ср|..." and `[$-F419]` "март|Ср|..." as in Excel.

## Context
Found by T-3666 (number-formats.md "LCIDs with no table of their own"). Both LCIDs now fall back to the first Russian table, ru-RU 0x419, which has "Март|Ср". Excel prints a lowercase month under both, and a lowercase weekday under 0x819 only. Probe the other ru-MD forms (`mmm`, `mmmmm`, `dddd`, every month and weekday) and other 0x3D-0x3F Russian tags (F819, FC19) before adding tables. Code: locale/excel_date_names.rs (tables), formatter/locale_tag.rs `by_language`.
  Owns probes/t3666-lcid-unknown-sublanguage.case.json S!A17 and S!A91.

## Acceptance criteria
- [ ] probes/t3666-lcid-unknown-sublanguage S!A17 and S!A91 pass; a probe covers the other names.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
