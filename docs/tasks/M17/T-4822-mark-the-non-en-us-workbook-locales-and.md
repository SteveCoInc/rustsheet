---
id: T-4822
title: "Mark the non-en-US workbook locales and languages as unverified in the API until M17 exits"
milestone: M17
lane: api
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-4808]
---

## Goal
Until M17 exits, the API, bindings and docs say plainly that the workbook locales and formula languages other than en-US are not verified against Excel, in the form T-4808 decides (documentation and a warning, or refusing them).

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. `get_supported_locales()` (engine and Python bindings) lists all six with no caveat today, and `Model::new_empty(.., locale, .., language)` accepts any of them. Users can therefore run a de or fr workbook with no evidence behind it.

## Acceptance criteria
- [ ] The API docs, Python docstrings and README name en-US as the only Excel-verified locale and language.
- [ ] The interim behavior T-4808 rules on is implemented and tested.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
