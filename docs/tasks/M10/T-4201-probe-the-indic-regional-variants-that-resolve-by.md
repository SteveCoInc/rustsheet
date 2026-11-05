---
id: T-4201
title: "Probe the Indic regional variants that resolve by the language fallback (bn-BD 845, ta-LK 849, ne-IN 861, [$-bn], [$-ne-IN])"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`[$-845]`, `[$-849]`, `[$-861]`, the neutral `[$-45]`, `[$-49]`, `[$-61]` and BCP-47 `[$-bn]`, `[$-bn-BD]`, `[$-ta-LK]`,
`[$-ne-IN]` print Excel's names, evidenced by a probe, not by the inferred language fallback.

## Context
Since T-2959 (#3161) `LOCALES` (locale/excel_date_names.rs) holds bn-IN, ta-IN and ne-NP, so these tags fall back to
those names through `by_language` and `resolve_bcp47` (locale_tag.rs), and under calendar digits 10 and 8 to the Latin
Saka and Hebrew names. Before, they printed the no-locale en-US names. The fallback is inferred and has been wrong for
other regions (number-formats.md "Regional variants": de-AT, es-MX, zh-SG). Found by the excel-fidelity reviewer on
#3161. Probe `TEXT(45000.75,"[$-845]mmm|mmmm|ddd|dddd|AM/PM")` and the same for each tag above, with a morning
`AM/PM` and `[$-100845]`-style Saka cells, like probes/t2959-indic-lcid-names.

## Acceptance criteria
- [ ] A filled probe covers every tag in the goal; a tag whose names differ from its fallback gets its own table, or an
      owned off cell.
- [ ] number-formats.md "Indic names" records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Indic LCIDs other than these regional variants.

## Outcome
Filled probes/t4201-indic-regional-variants (142 cells, excel-graph) for 845, 849, 861, 45, 49, 61, bn, bn-BD, ta-LK
and ne-IN (12 months, AM, Saka and Hebrew digits for the LCIDs). 845, 849, 861 and their neutral/BCP-47 forms have their own tables, not the language fallback;
recorded in number-formats.md "Indic regional variants". The 102 off cells are owned by T-4292; the eval baseline records them as known failures of the new case. Neutral 49 equals ta-IN and 61 equals ne-NP bar one Saka cell.
