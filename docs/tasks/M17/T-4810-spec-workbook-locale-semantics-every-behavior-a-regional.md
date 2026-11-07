---
id: T-4810
title: "SPEC: workbook-locale semantics (every behavior a regional setting changes) and the eval harness locale dimension"
milestone: M17
lane: research
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4808]
---

## Goal
A spec in docs/specs/ that lists every behavior a workbook's regional setting changes in Excel, how RustSheet models a workbook locale and formula language, and how the eval harness carries a locale.

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. The inventory to cover, to be confirmed by probes: (1) typed input: decimal and group separators, date and time order, currency, percent, typed booleans; (2) text coercion: VALUE, DATEVALUE, TIMEVALUE, arithmetic on text, criteria; (3) displayed text: General and built-in number, date and currency formats; (4) formula language: localized function names, error and boolean names, `;` argument separator, decimal comma, array-constant separators, R1C1 letters; xlsx stores formulas and number formats locale-neutral, so this concerns user input and display only (Graph's `formulasLocal` shows the localized form); (5) format codes written in the locale (TEXT's localized letters, e.g. de-DE `JJJJ`, and separators inside codes); (6) locale-aware functions: DOLLAR, FIXED, NUMBERVALUE defaults, CELL("format"), INFO, collation and case mapping; (7) what does not change (format-code locale tags, calculation). Written as its own task per CLAUDE.md directive 4.

## Acceptance criteria
- [ ] docs/specs/SPEC-NNNN (template) covers the inventory, the API model of locale and language, and the harness's locale dimension.
- [ ] Each inventory item names the probe shape that will evidence it.
- [ ] Reviewed and approved like any spec.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
