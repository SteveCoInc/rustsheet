---
id: T-4978
title: "IRR iteration count chosen by training-set bounding boxes and a guess classifier"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IRR's iteration budget does not depend on a training-set rectangle of flow count, peak and guess or on a `guess == 0.1` classifier.

## Context
`financial_util.rs` (~315–455) picks 200 versus 50 iterations from a bounding box. `IRR(VSTACK(-50,SEQUENCE(53,1,1,0)),-0.9)` returns a root but appending a zero flow gives `#NUM!`; t3478-irr-r4-wide A206 fails because its peak lies outside the box.

Source: docs/research/excel-parity-findings-2026-11-07.md, "Audit findings in pre-existing code" item 3 (filed by T-4975). The audit's claims are unverified here: reproduce first.

## Acceptance criteria
- [ ] Both cited repros match Excel.
- [ ] No IRR eval regressions; any remaining rule is structural and evidenced.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
