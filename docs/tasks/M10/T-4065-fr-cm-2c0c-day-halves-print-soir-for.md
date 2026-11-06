---
id: T-4065
title: "fr-CM [2C0C] day halves print soir for AM/PM, A/P and a/p"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(45000.75,"[$-2C0C]h AM/PM|h A/P|h a/p")` is "6 soir|6 soir|6 soir" as in Excel.

## Context
Found by T-3666/T-4480 (number-formats.md "LCIDs with no table of their own"). fr-CM 0x2C0C is an assigned LCID, so it falls back to fr-FR, whose day halves are "PM". Excel prints "soir". Probe the morning string and the other African French LCIDs (0x280C fr-SN, 0x300C fr-CI, 0x340C fr-ML, 0x380C fr-MA, 0x240C fr-CD, 0x3C0C fr-HT) before adding a table. Code: locale/excel_date_names.rs, formatter/locale_tag.rs.
  Owns probes/t4480-lcid-sublanguage-fallback.case.json S!A9.

## Acceptance criteria
- [x] probes/t4480-lcid-sublanguage-fallback S!A9 passes; a probe covers the morning string and the neighbouring LCIDs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed day halves and names under 18 French tags: probes/t4065-french-regional-halves (54 cells)
and t4065-french-regional-names (48 cells; excel-graph 2026-11-06). fr-CM (`[$-2C0C]`, `[$-fr-CM]`)
has the fr-FR names with "mat."/"soir" for every day-half form. The probe also found fr-MA
(`[$-380C]`, own short months) and fr-029 (`[$-1C0C]`, capitalised months), both added
(locale/variant_date_names.rs); the other regional French LCIDs print as fr-FR.
probes/t4480-lcid-sublanguage-fallback A9 passes; both new cases pass in full; 0 regressions.

Since T-4508 these names come from the generated locale tables; locale/variant_date_names.rs is gone.
