---
id: T-4042
title: "Name and day-half tables for LCIDs Excel resolves but that have none here"
milestone: M10
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every LCID that Excel resolves to a locale (`lcid_locales::has_locale`) prints that locale's month, day and
day-half names, also where its language's first table differs from it (`[$-1409]h AM/PM` is "6 pm", not "6 PM").

## Context
Found by T-3666. Since T-3666 an LCID with a locale but no table of its own falls back to its language's
first table, which is right only where the names agree. The two probes list where they do not:
- probes/t3666-lcid-sublanguage-sweep (8 languages x 64 sublanguages, column J extra LCIDs): English regional
  "am"/"pm" (0x1409, 0x1809, 0x1C09, 0x2009, 0x2809, 0x2C09, 0x3009, 0x3409, 0x3C09, 0x4809), fr-029 "Mars",
  fr-CM "soir", the Spanish regional "p. m." variants and "Marzo", `[$-A]` and 0xF80A "p. m.", 0xFF "mbooy".
- probes/t3666-lcid-locale-map (languages 0x01-0xFF x sublanguages 0-63, "A/P mmmm" of 0.75): every row whose
  language has no table here, and the regional variants of those that have one.
Semantics: number-formats.md "Which LCIDs have a locale". Code: formatter/locale_tag.rs `resolve_lcid`,
`by_language`; tables in locale/excel_date_names.rs (`LOCALES`). The day-half strings and names can be read off
both probes; `ddd`/`dddd` of the languages without a table need one more probe.

Owned cells:
probes/t3666-lcid-sublanguage-sweep
probes/t3666-lcid-locale-map

## Acceptance criteria
- [x] probes/t3666-lcid-sublanguage-sweep and probes/t3666-lcid-locale-map pass (or what is left is owned by a
      narrower follow-up).
- [x] `make check` green; reviewers approve.

## Out of scope
- Which LCIDs have a locale (settled by T-3666).

## Outcome
Done in the same branch as T-3666, by replacing every hand-written name table with Excel's own data rather than adding
regional tables one by one. Two more probes read Excel's 38 names and day halves under all 853 LCIDs with a locale
(probes/t3666-locale-names) and under all 1122 BCP-47 tags of [MS-LCID] 2.2 and CLDR (probes/t3666-locale-tag-names);
`rustsheet-excel-locales` (tools/) turns them into locale/excel_names.tsv (335 distinct sets), excel_lcids.tsv (845
LCIDs) and excel_tags.tsv (926 tags, with the LCID [MS-LCID] gives each), which `locale::excel_date_names` loads once.
`by_language`, `lcid_locales.rs`, regional_date_names.rs and the 48-entry `LOCALES` list are gone; the calendar name
sets (Hijri, lunar, Levant, Saka, Hebrew) are built from the tables (`calendar_date_names::calendar_names`). The tag
probe also settled extended hex past eight characters and the calendar byte's top bit (`calendar_digit`). Eval: both
sweeps pass (t3666-lcid-sublanguage-sweep 534/534, t3666-lcid-locale-map 255/255), as do the two new probes.
