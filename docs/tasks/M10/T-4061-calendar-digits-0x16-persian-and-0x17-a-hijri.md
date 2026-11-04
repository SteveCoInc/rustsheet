---
id: T-4061
title: "Calendar digits 0x16 (Persian) and 0x17 (a Hijri variant) of a locale tag"
milestone: M10
lane: functions-text
status: todo
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
- [ ] probes/t3666-calendar-top-bit A19:A22 pass; a probe covers the Persian calendar's months and leap years.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other calendar digits.
