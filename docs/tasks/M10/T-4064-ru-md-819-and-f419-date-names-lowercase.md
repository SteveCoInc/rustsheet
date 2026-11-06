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
Probed every month and weekday under ru-MD (`[$-819]`, `[$-ru-MD]`), the Russian high sublanguages
and ru-RU: probes/t4064-ru-md-names (72 cells, excel-graph 2026-11-06). ru-MD gets its own table
(dotted short months, lowercase months and short weekdays; locale/variant_date_names.rs), and
`[$-F419]`, `[$-F819]`, `[$-FC19]` get the month-case variants Excel prints (lowercase nominative,
capitalised genitive, lowercase genitive) through `SUBLANGUAGE_VARIANTS`, which `locale_tag::table`
reads after `LOCALES`. probes/t4480-lcid-sublanguage-fallback A17 and A91 pass; the new case passes
in full; 0 regressions.
