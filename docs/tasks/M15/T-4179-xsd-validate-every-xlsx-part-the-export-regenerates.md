---
id: T-4179
title: "XSD-validate every xlsx part the export regenerates against the ECMA-376 transitional and MS extension schemas"
milestone: M15
lane: harness
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-4172]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
Every part the export regenerates validates against the ECMA-376 transitional schemas plus the Microsoft extension
schemas, in the round-trip suites, catching element-order and attribute errors that Excel silently "repairs".

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §4, "Opening cleanly". Check the schema licences (ECMA-376 is freely available; record the terms) and
fetch rather than vendor if needed. A Rust XSD validator may not exist, so a tools/ (Python, lxml) validator called by
the harness is acceptable if it does no xlsx value parsing (CLAUDE.md conventions).

## Acceptance criteria
- [ ] The round-trip suites report XSD violations per part, at 0 for the fixtures suite, or each existing violation
      filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Validating parts we replay verbatim.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
