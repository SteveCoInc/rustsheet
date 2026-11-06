---
id: T-4480
title: "LCID language fallback: assigned LCIDs and sublanguages 0x3D-0x3F fall back to the language (45 t3666 probe cells main misses)"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Hex LCIDs with no table of their own fall back to their language exactly where Excel does: for every LCID Windows assigns and every sublanguage 0x3D-0x3F, and nowhere else.

## Context
T-3666 (#3119) bounded the sublanguages of four languages by hand (`sublanguage_defined`) and kept every other sublanguage as defined, with English exempt. A 98-cell probe made under T-3666 in a parallel session (probes/t4480-lcid-sublanguage-fallback, excel-graph 2026-11-04) sweeps sublanguages 0x20-0x3F for Korean and English, and unassigned and assigned sublanguages of nine languages. On main, 45 of its cells fail: the high sublanguages of Korean, German, Japanese, French, Chinese and Russian fall back in Excel, and the 0x20-0x3C sweeps print English. Semantics: number-formats.md "More regional variants". Code: formatter/locale_tag.rs `by_language`, `resolve_extended`.

## Acceptance criteria
- [x] probes/t4480-lcid-sublanguage-fallback passes but for the cells T-4064 and T-4065 own; text-locale-invalid-sublanguage, text-locale-regional-lcids-2 and t1924-locale-tag-extended-hex still pass in full.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`by_language` falls back to the language only for an LCID in [MS-LCID] 2.2 (new
`locale/lcids.rs`, 418 IDs, a unit test for size and order) or with sublanguage 0x3D-0x3F; this
replaces the hand-bounded `sublanguage_defined`. Extended-hex tags resolve like hex ones, so
`resolve_extended` and its English exemption are gone. number-formats.md's "Invalid sublanguage"
entry becomes "LCIDs with no table of their own". Eval: probes known failures -45 on the new case
(95/98 pass; A9, A17, A91 owned by T-4064 and T-4065); text-locale-invalid-sublanguage 96/96,
text-locale-regional-lcids-2 294/294, t1924 60/60; 0 regressions.
