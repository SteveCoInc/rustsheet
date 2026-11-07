---
id: T-4175
title: "Implement calculated items and solve order"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4505, T-4174, T-4792]
---

## Goal
Calculated items are evaluated per record over items of one field, with Excel's solve order and restrictions.

## Context
SPEC-0032; LibreOffice does not support them (libreoffice.md), so evidence is oracle-only. Excel for the web refuses or ignores every calculated item the case builder
writes (pivots.md "Excel for the web refuses calculated items", T-4130, T-4505), so no rule here has evidence
until T-4792 finds an Excel-saved file the web computes. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] p-calc item probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
