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
- [ ] A probe case covers the LCIDs; the rule is in number-formats.md; text-locale-regional-lcids-2 M1:M21 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 24 plain hex LCIDs (probes/text-locale-invalid-sublanguage, excel-graph). An LCID whose sublanguage is undefined for its language prints English with letter-kept day halves; defined sublanguages keep the language fallback; English (0xF409) stays en-US. Added `sublanguage_defined` (bounds Korean, Japanese, German, Dutch). Rule in number-formats.md. Eval: text-locale-regional-lcids-2 M1:M21 (21 cells) now pass; the new case adds 96 cells, 93 pass; J3, K3, L3 (lowercase halves of 1809/2809/3C09) owned by T-4066. Unprobed languages' sublanguages are still taken as defined.
