---
id: T-4066
title: "Lowercase day halves for en-IE, en-BZ and en-HK hex LCIDs (1809 2809 3C09)"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`[$-1809]`, `[$-2809]` and `[$-3C09]` print lowercase day halves ("6 pm|6 pm|6 pm") as Excel does.

## Context
probes/text-locale-invalid-sublanguage S!J3, S!K3, S!L3 (found by T-3666); ours prints "6 PM". Code: formatter/locale_tag.rs LOCALES tables.

## Acceptance criteria
- [x] S!J3, S!K3, S!L3 pass; `make check` green.

## Out of scope
-

## Outcome
Added `EN_LOWER_HALVES` (en-US names, halves "am"/"pm") and the LOCALES entries 0x1809 en-IE, 0x2809 en-BZ and
0x3C09 en-HK (locale/excel_date_names.rs). Eval: probes/text-locale-invalid-sublanguage S!J3, S!K3, S!L3 pass
(`eval/baseline/probes.txt` -3); no regressions. Side effect: the BCP-47 tags `en-IE`, `en-BZ`, `en-HK` now resolve
to these LCIDs (before: the en fallback, uppercase), noted in number-formats.md as unprobed.
