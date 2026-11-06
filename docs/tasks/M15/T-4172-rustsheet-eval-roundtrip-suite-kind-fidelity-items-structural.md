---
id: T-4172
title: "rustsheet-eval roundtrip suite kind: fidelity items, structural XML differ, package validator, baselines"
milestone: M15
lane: harness
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-4167]
---

## Goal
`rustsheet-eval` runs `kind = "roundtrip"` suites. Each case is loaded, saved under R0 and the standard R1 edits,
validated as a package, diffed structurally and scored per fidelity item. Known losses go to a baseline that only
shrinks, as eval baselines do.

## Context
Implements the round-trip contract spec (T-4167). Plan: docs/research/xlsx-round-trip-2026-11-04.md §6.1. The census prototype in
docs/research/xlsx-round-trip/ shows the shape and its limits: no sheet matching, element counts only. Reuse the
harness's suite, baseline and report machinery; do not add a second one (directive 3). The package validator checks
content types, relationship targets, unique rIds and dangling `r:id`s.

## Acceptance criteria
- [ ] A roundtrip suite over the fixtures workbooks runs in `make eval` and reports fidelity items by feature.
- [ ] `update-baseline` refuses a new loss.
- [ ] The validator fails a crafted package with a dangling `r:id`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The external corpora (T-4173).
- XSD validation (T-4179).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
