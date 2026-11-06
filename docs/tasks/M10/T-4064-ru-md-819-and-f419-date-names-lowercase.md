---
id: T-4064
title: "ru-MD [819] and [F419] date names: lowercase month (and weekday under 819) where ru-RU capitalises"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(45000.75,"[$-819]mmmm|ddd|...")` is "март|ср|..." and `[$-F419]` "март|Ср|..." as in Excel.

## Context
Found by T-3666/T-4480 (number-formats.md "LCIDs with no table of their own"). Both LCIDs now fall back to the first Russian table, ru-RU 0x419, which has "Март|Ср". Excel prints a lowercase month under both, and a lowercase weekday under 0x819 only. Probe the other ru-MD forms (`mmm`, `mmmmm`, `dddd`, every month and weekday) and other 0x3D-0x3F Russian tags (F819, FC19) before adding tables. Code: locale/excel_date_names.rs (tables), formatter/locale_tag.rs `by_language`.
  Owns probes/t4480-lcid-sublanguage-fallback.case.json S!A17 and S!A91.

## Acceptance criteria
- [x] probes/t4480-lcid-sublanguage-fallback S!A17 and S!A91 pass; a probe covers the other names.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done by T-4508: the generated locale tables hold Excel's names and day halves for every LCID, so probes/t4480-lcid-sublanguage-fallback S!A17 and S!A91 pass;
probes/t3666-locale-names (T-4507) covers every name and day half of the LCID and its neighbours.
