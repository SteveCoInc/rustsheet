---
id: T-4042
title: "Name and day-half tables for LCIDs Excel resolves but that have none here"
milestone: M10
lane: functions-text
status: todo
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
- [ ] probes/t3666-lcid-sublanguage-sweep and probes/t3666-lcid-locale-map pass (or what is left is owned by a
      narrower follow-up).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which LCIDs have a locale (settled by T-3666).
