---
id: T-4817
title: "Workbook locale: typed input and text coercion (separators, date order, currency, VALUE/DATEVALUE/TIMEVALUE) per locale"
milestone: M17
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4812, T-4813, T-4814, T-4815, T-4816]
---

## Goal
Typed input (`set_user_input`) and text coercion (VALUE, DATEVALUE, TIMEVALUE, arithmetic and criteria on text) follow the workbook locale as the M17 probes show: decimal and group separators, date and time order, currency and percent forms, and typed booleans.

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. Implements the T-4810 spec's area from the M17 probes (T-4812 to T-4816), which own its visibly-off cells; split per locale if one locale's evidence lands well before the others. Keep en-US unchanged: every en-US suite at 0 regressions.

## Acceptance criteria
- [ ] Every visibly-off cell of the area's M17 probes passes, or is owned by a narrower follow-up.
- [ ] docs/excel-semantics/ records the rules; en-US suites show 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
