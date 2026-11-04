---
id: T-4081
title: "Phase 1 exit and make eval measure Gate A (accuracy.json, a gate A line)"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The Phase 1 exit verdict (`make phase1-exit`, the dashboard's exit line) reports Gate A (decision 0188 R1(a)), and
the steward's `make eval` measures it, so the exit's furthest-off condition is visible on every run.

## Context
- `scripts/phase1_exit.py` had no Gate A line. docs/progress.md showed "Gate A: n/a" because only `make check`
  writes eval/runs/accuracy.json, and the steward runs `make eval`.
- On 2026-11-04 (92a5d396), `rustsheet-eval accuracy` reported 14,354 points, 0 failing, 118 known failures and
  199 missing tables. Gate A passes `make check` and does not hold.
- Decision 0268.

## Acceptance criteria
- [x] `make eval` writes eval/runs/accuracy.json; a failing point does not fail it; a crash leaves no JSON.
- [x] `phase1_exit.py` prints a `gate A` line: met when `holds`, otherwise not met with the counts; `n/a` when the
      JSON is absent, unreadable or older than the report. Tests in scripts/test_phase1_exit.py.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing the 199 missing tables (the harness Gate A slices and the M9 classification tasks T-3797 to T-3800).

## Outcome
- `make eval` runs the accuracy check after the suites and prints its summary line. `phase1_exit.py` gains the
  `gate A` line. At 92a5d396 it reads `gate A  not met  14354 points, 14236 inside, 118 known failures, 0 failing,
  0 list problems; missing tables 199`.
- Eval delta: none (no engine or suite change).
