---
id: T-4508
title: "Date names from probe-generated locale tables; drop the per-language sublanguage bounds and Indic name tables"
milestone: M10
lane: functions-text
status: done
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
locale/indic_date_names.rs) and falls back to a language's names for the LCIDs Windows assigns (`locale/lcids.rs`, T-4480). The unmerged branch of
T-4507 generated the tables with a tool (`rustsheet-excel-locales`) and passed every probe of main's that names a
locale. number-formats.md "Which LCIDs have a locale", "Calendar names per locale".

## Owned cases
Every visibly-off cell of these probes, or of the cells named (decision 0245):
- probes/t3666-calendar-top-bit
- probes/t3666-lcid-locale-map
- probes/t3666-lcid-sublanguage-sweep
- probes/t3666-locale-names
- probes/t3666-locale-tag-names
- probes/t3666-review-followups A1:A6 (tags after `B2`; the comma cells A7:A12 are T-4515's)
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
- [x] The locale and calendar tables are generated from the probes by a tool, and a test checks they are current.
- [x] The language fallback (`by_language`, locale/lcids.rs) and the per-language name tables are gone; no regression in any suite.
- [x] The t3666-* and t4061-* probes pass (calendar digits 0x16 and 0x17 included).
- [x] `make check` green; reviewers approve.

## Out of scope
- Fraction and comma rules (T-4515).

## Outcome
Done. The locale and calendar names are data: `rustsheet-excel-locales` (tools/) generates locale/excel_names.tsv,
excel_lcids.tsv, excel_tags.tsv and excel_calendars.tsv, and the Persian and Um al-Qura tables
(formatter/persian_years.txt, um_al_qura_months.txt), from the T-4507 probes alone; tools/tests/test_excel_locales.py
checks the committed files are its output. `locale_tag` resolves a tag in one order (hex LCID with a locale, BCP-47
tag, hex LCID without one, extended hex) and reads the calendar from one table of digits (`calendar_spec`). Gone:
the hand-written name tables (locale/regional_date_names.rs, indic_date_names.rs, calendar_date_names.rs and the
tables in excel_date_names.rs), the language fallback (`by_language`, locale/lcids.rs) and the per-calendar special
cases (`hebrew_or_saka`, `hijri_names`). Eval: probes known failures fall by 7180 cells, 0 regressions in any
suite; every t3666-* and t4061-* cell passes except the comma cells of t3666-review-followups (T-4515), and so do
main's per-language probes (t2959-indic-lcid-names, t4201-indic-regional-variants, t4480-lcid-sublanguage-fallback,
text-locale-invalid-sublanguage). T-4064 and T-4065, fixed on main by hand in #3538 (locale/variant_date_names.rs) while this was in review, now come from the tables too, so that file is gone as well; #3538's comma-suffix reading (`numeral_suffix`, a BCP-47 locale before the comma) is kept in `resolve_with` and `numerals`.
Review round 3: the [MS-LCID] tag list (tools/src/rustsheet_tools/ms_lcid_tags.tsv) missed 8 rows of the spec (so,
ff-NG, ff-Latn-NG, quc-Latn-GT, az-Cyrl-AZ, uz-Cyrl-UZ, mn-Mong-CN, en-029, es-419), so those tags lost their LCID's
calendar names after `B2`. With the rows added, probes/t4508-ms-lcid-tags-after-b2 (27 cells, excel-graph 2026-11-06)
shows each tag after `B2` printing what its LCID prints, and the three [MS-LCID] tags the tag probe never tried (ff-NG,
quc-Latn-GT, en-029) printing their LCID's names, which the generator now takes from it (`confirmed_tags`). All 27 pass.
