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
Done (general rule). Swept every language 0x01-0xFF x sublanguage 0-63 (probes/t3666-lcid-locale-map, 255 cells of
64 TEXT calls each) and 8 languages in detail (probes/t3666-lcid-sublanguage-sweep, 534 cells): whether an LCID has a
locale is a set Excel takes from Windows (MS-LCID's LCIDs, less 0x1 and 0x2, plus the 0x3D-0x3F aliases), now the
LCID table locale/excel_lcids.tsv (T-4042): an LCID outside it has no locale, `resolve_extended` is gone (extended hex
follows the same rule), and 0xC00 and 0x7F are en-US.
number-formats.md "Which LCIDs have a locale". Eval: text-locale-regional-lcids-2 S!M1:M21 and
t1924-locale-tag-extended-hex S!B22 pass. T-4042, done in the same branch, replaced the name tables with Excel's data for every LCID and tag, so these sweeps pass too.
