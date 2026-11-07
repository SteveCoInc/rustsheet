---
id: T-4808
title: "Decision: workbook locales beyond en-US as milestone M17 (scope, locales, phase, interim API status)"
milestone: M17
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A decision record that brings workbook locales beyond en-US into scope as milestone M17: which locales (en-GB, de-DE, fr-FR, es-ES and it-IT, the ones the engine ships), where M17 sits in the phase order, how evidence is gathered (one oracle site per locale), and what the API says about those locales until M17 exits.

## Context
The engine ships six workbook locales (`locale/locales.bin`, CLDR: en, en-GB, de, es, fr, it) and five formula languages (`language/language.bin`: en, it, fr, de, es), inherited from IronCalc. Only en-US has Excel evidence: the parity bar is en-US Excel (CLAUDE.md), the eval harness runs every case under `"en"`, and the oracle's locale is a SharePoint site setting pinned to en-US by its canary (docs/architecture/oracle.md, "Locale is a site setting"; decision 0030 rule 2, decision 0032). Format-code locale tags (`[$-40C]`) are a separate, already covered axis: 841 LCIDs and 935 BCP-47 tags with Excel-generated name tables (T-4508). This milestone covers the workbook's own regional setting. Prime directive 1 and the milestone table make en-US the bar, so widening it needs a decision record (`make new-decision`) and an owner ruling, which the owner asked for on 2026-11-07. The record adds M17 to docs/milestones.md (scope, exit criteria measured per locale like M10's) and the phase table, and settles whether the five locales stay callable meanwhile (T-4822).

## Acceptance criteria
- [ ] The decision record is approved and docs/milestones.md has an M17 section and a phase-table row.
- [ ] It names the locales, the evidence source per locale and M17's exit criteria (per-locale probe rows at visible parity).
- [ ] It rules on the interim API status of the non-en-US locales (T-4822).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
