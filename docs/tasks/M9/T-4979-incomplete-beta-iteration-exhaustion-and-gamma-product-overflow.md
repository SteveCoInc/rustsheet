---
id: T-4979
title: "Incomplete-beta iteration exhaustion and gamma-product overflow return wrong finite values"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An incomplete-beta call that runs out of iterations returns an error instead of a finite probability, and a gamma-product overflow no longer zeroes a representable beta tail.

## Context
The audit reports both behaviours in the incomplete-beta kernel; repro inputs were not recorded in the findings, so the first step is to find them.

Source: docs/research/excel-parity-findings-2026-11-07.md, "Audit findings in pre-existing code" item 5 (filed by T-4975). The audit's claims are unverified here: reproduce first.

## Acceptance criteria
- [ ] A test per behaviour with Excel's value (or `#NUM!` with evidence).
- [ ] No eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
