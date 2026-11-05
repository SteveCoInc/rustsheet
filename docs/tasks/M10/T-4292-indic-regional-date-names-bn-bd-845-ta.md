---
id: T-4292
title: "Indic regional date names: bn-BD 845, ta-LK 849, ne-IN 861 and the neutral 45, 49, 61 have their own tables"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
`TEXT` with `[$-845]`, `[$-849]`, `[$-861]`, `[$-45]`, `[$-49]`, `[$-61]`, `[$-bn]`, `[$-bn-BD]`, `[$-ta-LK]` and
`[$-ne-IN]` prints Excel's own names for each, not the bn-IN, ta-IN and ne-NP fallback.

## Context
Probe probes/t4201-indic-regional-variants (T-4201, excel-graph, 2026-11-05) shows the fallback is wrong. Our engine
prints the bn-IN/ta-IN/ne-NP tables for these tags; the case's 102 off cells are this task's. Excel's results:
bn-BD 845 (and neutral 45, `bn`, `bn-BD`) has full month names for `mmm` too ("মার্চ|মার্চ"), short weekdays without
a trailing dot ("বুধ", "সোম"), and day halves "AM"/"PM", unlike bn-IN. ta-LK 849 (and `ta-LK`) is its own table: dotted short months and days
("ஜன.|ஜனவரி", "ஞாயி.|ஞாயிறு") and halves "முற்பகல்"/"பிற்பகல்", unlike ta-IN's "காலை"/"மாலை"; neutral 49 is a third table ("மார்ச்|மார்ச்|புதன்|புதன்கிழமை|மாலை", halves "காலை"/"மாலை", full
weekday "ஞாயிற்றுக்கிழமை"). ne-IN 861 (and `ne-IN`) differs from ne-NP in the weekday names ("आइतबार",
"बुधबार") and neutral 61 is like ne-NP ("आइतवार", "बुधवार"). Under calendar digits 10 and 8 every tag prints the Latin
Saka and Hebrew names, except 861 and 61 whose Saka cells print native names ("फल्गुण", weekday "बुधवार"). The
off cells are in the case file; `make eval-show CASE=probes/t4201-indic-regional-variants` lists them. Code:
locale/excel_date_names.rs (`LOCALES`, `by_language`), formatter/locale_tag.rs (`resolve_bcp47`),
locale/indic_date_names.rs.

## Acceptance criteria
- [ ] Every cell of probes/t4201-indic-regional-variants passes; eval baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other Indic LCIDs and regions.
