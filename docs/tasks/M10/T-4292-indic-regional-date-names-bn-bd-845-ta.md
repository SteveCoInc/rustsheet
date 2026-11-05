---
id: T-4292
title: "Indic regional date names: bn-BD 845 (and 45), ta-LK 849, ne-IN 861 and the neutral 61 Saka row have their own names"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
`TEXT` with `[$-845]`, `[$-45]`, `[$-bn]`, `[$-bn-BD]`, `[$-849]`, `[$-ta-LK]`, `[$-861]`, `[$-ne-IN]` and `[$-61]` under
Saka digits prints Excel's own names, not the bn-IN, ta-IN and ne-NP fallback.

## Context
Probe probes/t4201-indic-regional-variants (T-4201, excel-graph, 2026-11-05); its 102 off cells are this task's, listed by
`make eval-show CASE=probes/t4201-indic-regional-variants.case.json`. Neutral `[$-49]` already equals ta-IN (no change).
Wanted: bn-BD 845/45/bn/bn-BD, own table (full month names for `mmm`, undotted short days); ta-LK 849/ta-LK, own table
("ஜன.|ஜனவரி", "முற்பகல்"/"பிற்பகல்"); ne-IN 861/ne-IN, own months and weekdays ("जनवरी", "बुधबार"), native Saka names;
neutral 61 equals ne-NP except one Saka cell (native names, F14). Details in number-formats.md "Indic regional variants".
Code: locale/excel_date_names.rs (`LOCALES`, `by_language`), formatter/locale_tag.rs (`resolve_bcp47`),
locale/indic_date_names.rs.

## Acceptance criteria
- [ ] Every cell of probes/t4201-indic-regional-variants passes; eval baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other Indic LCIDs and regions.
