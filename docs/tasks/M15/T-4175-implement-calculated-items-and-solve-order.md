---
id: T-4175
title: "Implement calculated items and solve order"
milestone: M15
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4173, T-4174]
---

## Goal
Calculated items are evaluated per record over items of one field, with Excel's solve order and restrictions.

## Context
SPEC-0032; LibreOffice does not support them (libreoffice.md), so evidence is oracle-only. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] p-calc item probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
