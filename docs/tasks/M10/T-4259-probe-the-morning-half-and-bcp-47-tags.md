---
id: T-4259
title: "Probe the morning half and BCP-47 tags of en-IE, en-BZ and en-HK"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`[$-1809]`, `[$-2809]` and `[$-3C09]` (en-IE, en-BZ, en-HK) match Excel for the morning half and for their BCP-47 spellings, not only the 18:00 afternoon half.

## Context
T-4066 added `EN_LOWER_HALVES` (locale/excel_date_names.rs) from probes/text-locale-invalid-sublanguage S!J3, S!K3, S!L3 (18:00 only). Unprobed and extrapolated: the morning half `am` (`TEXT(0.25,"[$-1809]h AM/PM|h A/P|h a/p")`) and `[$-en-IE]`, `[$-en-BZ]`, `[$-en-HK]`, which now resolve to the same LCIDs (before: the en fallback, uppercase). Use the `excel-probe` skill; notes in docs/excel-semantics/number-formats.md ("Lowercase day halves").

## Acceptance criteria
- [ ] Probe cells for the morning half of the three LCIDs and the three BCP-47 tags exist with oracle values.
- [ ] Ours matches them (fix `EN_LOWER_HALVES` or the LOCALES rows if not); number-formats.md loses "unprobed".
- [ ] `make check` green; reviewers approve.

## Out of scope
-
