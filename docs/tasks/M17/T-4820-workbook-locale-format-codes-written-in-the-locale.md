---
id: T-4820
title: "Workbook locale: format codes written in the locale (TEXT's localized letters and separators)"
milestone: M17
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4812, T-4813, T-4814, T-4815, T-4816]
---

## Goal
Format codes written in the locale, as a user types them in TEXT under that locale (de-DE `TT.MM.JJJJ`, decimal comma inside codes), read as Excel reads them; codes stored in xlsx stay en-US.

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. Implements the T-4810 spec's area from the M17 probes (T-4812 to T-4816), which own its visibly-off cells; split per locale if one locale's evidence lands well before the others. Keep en-US unchanged: every en-US suite at 0 regressions.

## Acceptance criteria
- [ ] Every visibly-off cell of the area's M17 probes passes, or is owned by a narrower follow-up.
- [ ] docs/excel-semantics/ records the rules; en-US suites show 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
