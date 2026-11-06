---
id: T-4508
title: "Date names from probe-generated locale tables; drop the per-language sublanguage bounds and Indic name tables"
milestone: M10
lane: functions-text
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every LCID and BCP-47 tag Excel resolves prints Excel's own date names and day halves, under every calendar digit, from
tables generated from the T-4507 probes, so no language needs code of its own.

## Context
probes/t3666-* and t4061-* (T-4507) hold Excel's names for all 853 LCIDs of the locale map, 1122 tags and every calendar
digit. Main covers the languages probed one by one (hand-written tables in locale/excel_date_names.rs,
locale/indic_date_names.rs) and bounds four languages' sublanguages (`sublanguage_defined`). The unmerged branch of
T-4507 generated the tables with a tool (`rustsheet-excel-locales`) and passed every probe of main's that names a
locale. number-formats.md "Which LCIDs have a locale", "Calendar names per locale".

## Owned cases
Every visibly-off cell of these probes (decision 0245):
- probes/t3666-calendar-top-bit
- probes/t3666-lcid-locale-map
- probes/t3666-lcid-sublanguage-sweep
- probes/t3666-locale-names
- probes/t3666-locale-tag-names
- probes/t3666-review-followups
- probes/t4061-calendar-names-all
- probes/t4061-calendar-names-by-language
- probes/t4061-calendar-names-digest
- probes/t4061-calendar-names-full
- probes/t4061-calendars-without-locale
- probes/t4061-hijri-variant-dates
- probes/t4061-persian-and-hijri-variant
- probes/t4061-persian-new-years
- probes/t4061-review-followups
- probes/t4061-umalqura-months

## Acceptance criteria
- [ ] The locale and calendar tables are generated from the probes by a tool, and a test checks they are current.
- [ ] `sublanguage_defined` and the per-language name tables are gone; no regression in any suite.
- [ ] The t3666-* and t4061-* probes pass (calendar digits 0x16 and 0x17 included).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fraction and comma rules (T-4515).
