---
id: T-4061
title: "Calendar digits 0x16 (Persian) and 0x17 (a Hijri variant) of a locale tag"
milestone: M10
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Calendar digits 0x16 and 0x17 of a locale tag print what Excel prints: 0x16 the Persian (Solar Hijri) date and names,
0x17 its Hijri variant (`[$-16040C]yyyy-mm-dd|mmmm|ddd` of 45000 is "1401-12-24|ésfànd|Chaharchanbé").

## Context
Found by T-3666 (probes/t3666-calendar-top-bit A19:A22, `[$-16040C]`, `[$-160411]`, `[$-17040C]`, `[$-170411]`): we
print them in the locale's own Gregorian calendar. Under 0x16 fr-FR has French transliterations, ja-JP Persian script;
under 0x17 the date and months are digit 6's Hijri ones, but `ddd` prints the full weekday ("Mercredi") under fr-FR,
and ja-JP gets the English Hijri names ("Sha'ban|AlArbia'a"). The top bit is dropped (`calendar_digit`), so 0x96 and
0x97 behave the same. Probe both digits over more LCIDs (en-US, fa-IR 0x429, ar-SA, an LCID with no locale) and the
Persian month boundaries (Nowruz) before implementing. Semantics: number-formats.md "Calendars of a locale tag".
Code: formatter/locale_tag.rs `resolve_lcid` (TODO(T-4061)), formatter/calendar_dates.rs.

## Acceptance criteria
- [x] probes/t3666-calendar-top-bit A19:A22 pass; a probe covers the Persian calendar's months and leap years.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other calendar digits.

## Outcome
Done, with the calendar-name model generalized rather than the two digits added as cases. Probes (excel-graph,
2026-11-04): t4061-persian-new-years (the Persian new year of every Gregorian year 1900-9999), t4061-umalqura-months
(Um al-Qura month starts 1900-2100), t4061-hijri-variant-dates, t4061-persian-and-hijri-variant, and the name sweeps
t4061-calendar-names-digest/-full/-all/-by-language and t4061-calendars-without-locale. Findings: the digits are Windows
calendar ids (0x16 Persian, 0x17 Um al-Qura); Persian leap years follow no cycle (Excel's table of 8100 years); Um
al-Qura is a 1318-1500 AH month table with the digit-6 date outside; and every calendar's names are per-LCID data (32
Hijri name sets, 10 Persian, odd alias transforms), not a rule by language. Code: `locale_tag::CALENDARS` (one spec per
digit) and `localize` replace `resolve_lcid`'s match, `without_locale`, `hijri_names` and `hebrew_or_saka`;
locale/excel_calendars.tsv (generated) replaces calendar_date_names.rs; formatter/table_calendars.rs reads
persian_years.txt and um_al_qura_months.txt; a hex LCID with no locale is en-US/Gregorian with `A/P` as written. Eval:
t3666-calendar-top-bit A19:A22 and every new t4061 case pass; 0 regressions.
