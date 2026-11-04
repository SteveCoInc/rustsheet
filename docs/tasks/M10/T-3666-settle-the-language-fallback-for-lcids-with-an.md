---
id: T-3666
title: "Settle the language fallback for LCIDs with an unknown sublanguage (0x2412)"
milestone: M10
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`[$-2412]` and other plain hex LCIDs with an invalid sublanguage print what Excel prints (English for 0x2412), not the language table.

## Context
T-1875 found `[$-2412]` prints English in Excel but `by_language` in formatter/locale_tag.rs gives Korean (probes/text-locale-regional-lcids-2 S!M1:M21 fail; docs/excel-semantics/number-formats.md "More regional variants"). Also owns the t1924 cell T-1875 left (probes/t1924-locale-tag-extended-hex S!B22, `[$-_409]h A/P`); A2, A3, B2, B3, B10, B28 and B29 pass since T-3882 added the six name tables. Probe more sublanguage-invalid LCIDs (0x0812, 0x1412, 0x2411, 0x2407, 0xF409) to find the rule, then fix `by_language`.

## Acceptance criteria
- [x] A probe case covers the LCIDs; the rule is in number-formats.md; text-locale-regional-lcids-2 M1:M21 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Settled the rule with probes/t3666-lcid-unknown-sublanguage (98 cells, excel-graph 2026-11-04): an
LCID with no table of its own falls back to its language's first table only when Windows assigns
it ([MS-LCID] 2.2) or its sublanguage is 0x3D-0x3F. Every other sublanguage (unassigned below
0x20, and all of 0x20-0x3C) has no locale: en-US names and `A/P` as written. `by_language`
(formatter/locale_tag.rs) applies the rule with the new `locale::lcids::is_assigned` (the MS-LCID
list, copied verbatim, not fitted). Extended-hex tags now resolve the same way, so
`resolve_extended` and its sublanguage special case are gone. number-formats.md gains "LCIDs with
no table of their own". Eval: probes known failures -22 (text-locale-regional-lcids-2 M1:M21 and
t1924-locale-tag-extended-hex B22 pass; both cases now pass fully), new case 95/98, 0 regressions.
The 3 open cells are two regional quirks the probe found, filed as T-4064 (ru-MD and 0xF419
lowercase names; A17, A91) and T-4065 (fr-CM "soir"; A9). M10 probe row: -22 visibly off.
