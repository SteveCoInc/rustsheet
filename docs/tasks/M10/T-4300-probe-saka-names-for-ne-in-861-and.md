---
id: T-4300
title: "Probe Saka names for ne-IN 861 and neutral 61 (12 months, 7 days, AM/PM) and fix comment placement in hebrew_or_saka"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for every Saka month, weekday and day half under `[$-100861]`, `[$-100061]` and `[$-ne-IN]` digit 10, and the code matching it.

## Context
T-4292 (PR 3291) gave 0x861 and neutral 0x61 the hi-IN `SAKA_HINDI` table because probes/t4201-indic-regional-variants C14 and F14 show only Phalguna and Wednesday. The other 11 months, 6 days and AM/PM are inferred. The guard in `hebrew_or_saka` (formatter/locale_tag.rs) also sends any 0x?61 except 0x461 to native names, unprobed. Reviewer notes: the comment above the `0x61` arm sits on the wrong arm and the doc comment of `hebrew_or_saka` omits the exception; number-formats.md should say neutral `[$-45]` and `bn` resolve to bn-BD.

## Acceptance criteria
- [ ] A probe case with all 12 Saka months, 7 days and AM/PM for 861 and 61, oracle-filled from excel-graph; the engine matches or mismatches are filed.
- [ ] Comment placement and doc comment fixed; the `[$-45]`/`bn` rule added to number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
