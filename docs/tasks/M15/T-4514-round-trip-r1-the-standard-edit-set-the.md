---
id: T-4514
title: "Round-trip R1: the standard edit set, the expected-delta rule, idempotence and per-case exceptions (SPEC-0033 D1, D5, D8)"
milestone: M15
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-4509, T-4513]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
Round-trip suites run the R1 scenarios of SPEC-0033 D1 and check the expected-delta rule of D5, so "load, change one
number, save" is proven to change nothing else. The idempotence property and per-case exceptions with their
self-test (D8) land with them.

## Context
SPEC-0033 D1, D5 and D8. The edits E1–E6 apply on the first visible worksheet, through the facade's `edit`/`set`
(crates/rustsheet/src/edit.rs). `Δ(E)` takes its changed-value cells from the engine's two recalculations, not from a
diff. Idempotence means saving our own R0 output again gives an empty `P`. `exceptions.toml` holds narrow tokens,
each naming its decision or task, and a self-test fails on an exception that excuses nothing.

## Acceptance criteria
- [ ] The fixtures suite runs R0 and R1/E1–E6 and R1/all.
- [ ] A crafted export that also drops an unrelated element is caught as a delta violation.
- [ ] The idempotence check runs on every case.
- [ ] The exception self-test fails on a stale exception.
- [ ] The baseline is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- R2 (M4).
- Excel goldens (T-4180).
