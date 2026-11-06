---
id: T-4418
title: "lookup.md: move the 'Which arguments lift is inferred' sentence back under the COUNTIFS/AVERAGEIF/MAXIFS bullet"
milestone: M9
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In docs/excel-semantics/lookup.md, section "Statistical aggregates over a lifted OFFSET", the sentence that starts
"Which arguments lift is inferred from SUMIF/SUMIFS" belongs to the COUNTIFS, AVERAGEIF and MAXIFS bullet again.

## Context
- T-4396 (PR #3433) added a bullet for AVERAGEIFS and MINIFS between the COUNTIFS/AVERAGEIF/MAXIFS bullet and the
  continuation sentence "Which arguments lift is inferred from SUMIF/SUMIFS (...); forms with only some ranges lifted
  are unprobed (T-2833)". That sentence now reads as if it belonged to the AVERAGEIFS/MINIFS bullet, whose
  partial-lift forms are probed (probes/t4396-lift-*-first, -criteria). The excel-fidelity reviewer flagged this as
  a MINOR finding on #3433.
- Wait for #3433 to merge first. The text is not on main until then.

## Acceptance criteria
- [ ] The sentence sits under the COUNTIFS/AVERAGEIF/MAXIFS bullet, above the AVERAGEIFS/MINIFS bullet, with its
      wording unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to the rules themselves.
