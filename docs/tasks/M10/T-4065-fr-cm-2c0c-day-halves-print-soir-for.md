---
id: T-4065
title: "fr-CM [2C0C] day halves print soir for AM/PM, A/P and a/p"
milestone: M10
lane: functions-text
status: todo
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
- [ ] probes/t4480-lcid-sublanguage-fallback S!A9 passes; a probe covers the morning string and the neighbouring LCIDs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
