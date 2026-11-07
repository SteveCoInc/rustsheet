---
id: T-2959
title: "Date-name tables for LCIDs Excel resolves but we have no table for (bn-IN 445 prints the Latin Saka names)"
milestone: M10
lane: functions-text
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A `[$-...]` tag over an LCID Excel resolves but that has no name table here (bn-IN 0x445 first) prints Excel's names, not the no-locale en-US fallback.

## Context
`locale_tag.rs` treats an LCID with no table in `locale/excel_date_names.rs` `LOCALES` (and no language table) as having no
locale. Excel resolves more: `TEXT(45000,"[$-100445]mmm|mmmm|ddd|dddd")` is "Phalguna|Phalguna|Budh|Budhavara" (the
Latin Saka names), ours "Dec|December|Wed|Wednesday" (probes/t2779-hebrew-saka-names E18, T-2779). Probe 0x445's
Gregorian names (and other Indic LCIDs: 0x44E, 0x449, 0x44A, 0x44B, 0x457, 0x461) and add their tables, or a list of
LCIDs Excel resolves, so the calendar digits apply. number-formats.md "Hebrew (8) and Saka (10)" notes the miss.

## Acceptance criteria
- [x] probes/t2779-hebrew-saka-names E18 leaves eval/baseline/probes.txt, with a probe of the new tables.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Picked by a sonnet run; stopped before coding. The task needs oracle-evidenced Gregorian month/day names (native scripts) for
bn-IN 0x445 and 0x44E, 0x449, 0x44A, 0x44B, 0x457, 0x461, plus a decision on tables vs. a "resolved LCIDs" list in
`locale_tag.rs` / `locale/excel_date_names.rs`. No evidence exists in the repo yet (`rustsheet-oracle status`: excel-graph OK).
Next step: use the `excel-probe` skill to probe `TEXT(45000,"[$-<lcid>]mmm|mmmm|ddd|dddd")` for each LCID, add tables, then
remove E18 of probes/t2779-hebrew-saka-names from eval/baseline/probes.txt via `make eval-baseline`.

## Outcome
Probed the seven Indic LCIDs (probes/t2959-indic-lcid-names, 112 cells, excel-graph 2026-11-04): each has its own
native-script Gregorian names and day halves, and each resolves to a locale under calendar digits 10, 8 and 6 (Latin
Saka, Latin Hebrew, English Hijri names). Added their tables (locale/indic_date_names.rs) to `LOCALES`, so
`[$-100445]` now prints "Phalguna|Phalguna|Budh|Budhavara" (t2779-hebrew-saka-names E18 passes). Recorded in
number-formats.md "Indic names", including the kn-IN April quirk ("ಏಏಪ್ರಿಲ್"). Eval delta: probes +112 passing cells (new case), known failures -1 (t2779-hebrew-saka-names E18).
