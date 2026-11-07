---
id: T-4809
title: "Oracle: one Excel for the web site per workbook locale, each pinned by its own canary"
milestone: M17
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4808]
---

## Goal
`rustsheet-oracle fill` can compute a case under each M17 locale: one Excel for the web site per locale, configured with that regional setting, each pinned by its own canary, and a sidecar records the locale it was computed under.

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. The excel-graph backend calculates in its SharePoint site's regional settings and refuses to fill when the en-US canary fails, so each locale needs its own site (or another way to set the session's locale; research Graph's options first, e.g. whether a workbook session or `formulasLocal`/`valuesAsJson` can take a culture). **Owner action:** creating the sites and granting the oracle app access is an owner step; record what is needed in the task before blocking on it. Canary per locale: a date-order and decimal check whose en-US answer differs (`DATEVALUE` of a day-first date, `VALUE` with a decimal comma).

## Acceptance criteria
- [ ] Each M17 locale has an oracle target whose canary passes for that locale and fails for the others.
- [ ] `rustsheet-oracle fill` takes the locale (from the case or suite) and refuses a case whose locale has no target.
- [ ] Sidecars record the locale; docs/architecture/oracle.md documents the setup.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
