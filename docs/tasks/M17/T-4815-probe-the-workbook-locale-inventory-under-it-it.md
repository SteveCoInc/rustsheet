---
id: T-4815
title: "Probe the workbook-locale inventory under it-IT (typed input, coercion, display, formula language, TEXT codes, locale-aware functions)"
milestone: M17
lane: research
status: todo
priority: P2
size: M
tier: sonnet
depends_on: [T-4809, T-4811]
---

## Goal
Excel's answers under the it-IT workbook locale for every item of the T-4810 inventory, in probe cases filled by the it-IT oracle target, and recorded in docs/excel-semantics/.

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. One probe batch per inventory area (typed input, coercion, displayed text, formula language via `formulasLocal`, TEXT format codes in the locale, DOLLAR/FIXED/NUMBERVALUE/CELL/INFO, sorting), each case tagged `locale = "it-IT"` and M17. Every visibly-off cell gets an owner among T-4817 to T-4821 (decision 0245).

## Acceptance criteria
- [ ] Probe cases for every inventory area run under it-IT and are filled by its oracle target.
- [ ] The findings are recorded in docs/excel-semantics/ (a locale section per area).
- [ ] Every visibly-off cell is owned by an M17 implementation task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
