---
id: T-4173
title: "Round-trip suites over calipers (with init.xlsx), poi, corpus-bench and fixtures, with fidelity baselines"
milestone: M15
lane: harness
status: blocked
priority: P2
size: S
tier: sonnet
depends_on: [T-4172]
---

## Goal
Round-trip suites over Calipers (all goldens; `roundtrip/` scored against `init.xlsx` → Excel golden), poi,
corpus-bench and the fixtures, each with a fidelity baseline, so every later M15 change is measured.

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §6.2. The calipers fetch includes only `**/golden.xlsx` today (eval/suites/calipers/suite.toml), so widen
it to `init.xlsx` for the `roundtrip/` cases. Corpus-bench and calipers are not in `make check` for time reasons, so
follow their gating pattern.

## Acceptance criteria
- [ ] The four suites run and baselines are committed.
- [ ] The eval report shows the fidelity table by feature and corpus. Its first numbers are copied into the M15
      section of docs/status.md by the steward, not by this PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New corpora.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
