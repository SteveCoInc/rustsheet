---
id: T-4976
title: "IRR and XIRR over a huge blank range abort on allocation before the budget check"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`=IRR(B1:XFC1000000)` (and XIRR on a similar range) no longer aborts the process; it returns Excel's result or a budget `#NUM!`/DEVIATION within the per-call budget (CLAUDE.md directive 5).

## Context
The audit reports a SIGABRT allocation failure: `financial/mod.rs` collects every coordinate of the range before the lift cost is charged, while `SUM` on the same range is fine.

Source: docs/research/excel-parity-findings-2026-11-07.md, "Audit findings in pre-existing code" item 4 (filed by T-4975). The audit's claims are unverified here: reproduce first.

## Acceptance criteria
- [ ] A regression test evaluates `IRR` and `XIRR` over a full-sheet-scale blank range without aborting, within the existing work budget.
- [ ] The cost is charged (or the range is streamed) before any per-cell allocation.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
