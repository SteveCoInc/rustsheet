---
id: T-3968
title: "t3953 score: measure F.DIST.RT and FDIST against their own exact v, and cite T-3567's argument rule (T-3953 follow-up)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
docs/research/t3953/score.py measures columns A (F.DIST.RT) and B (FDIST) of probes-numeric
`t3953-f-tail-bfrac-tiny` against their own 50-digit reference, and statistics.md's T-3953 entry quotes the
corrected counts and cites T-3567's F argument rule.

## Context
- Excel-fidelity review of T-3953 (PR #3009, MINOR). score.py takes A's and B's reference from the row's column C
  math value. The math tool evaluates C's argument `d2/(d1t*X+d2)` in doubles, so that reference is the tail at a
  rounded v, not at the exact v = d2/(TRUNC(d1)·X + d2) with X the double of its literal. The two references round to
  different doubles on 806 of 1,200 rows (max 6.7E-15 relative).
- The reviewer's recount against the exact reference:
  - Excel F.DIST.RT, exact / 4 ulp / 15 digits: 12 / 72 / 180 (quoted: 8 / 75 / 180).
  - Engine: 16 / 88 / 237 (quoted: 13 / 87 / 235).
  - Excel p99: 1.2E-13 (quoted: 1.1E-13).
  - Engine high / low: 576 / 624 (quoted: 574 / 626).
  No conclusion changes.
- The entry's first bullet leaves the 185 rows where F.DIST.RT differs from the sheet-v BETA.DIST to T-3954. It does
  not cite "F's incomplete-beta arguments keep r >= 1 (T-3567)" in statistics.md, the established argument model
  (fl(1/fl(1 + r)), r = fl(mx/n)), which likely explains them.
- The generator's `branch()` mirrors `bratio::dispatch` by hand (design review MINOR). Re-check it against the
  engine's dispatch when touching the script.

## Acceptance criteria
- [ ] score.py computes A's and B's reference with mpmath (dps >= 50) from (the double of X, TRUNC(d1), d2) and keeps
      C's and D's from the math sidecar. score.out is regenerated.
- [ ] statistics.md's "F right tail on bfrac at a tiny kernel argument" entry and T-3953's Outcome quote the new
      counts. The entry cross-references "F's incomplete-beta arguments keep r >= 1 (T-3567)".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change or fit (T-3954).

## Deferred by decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) parks this task until Phase 1 exits. It is work on the incomplete-beta family whose cells are first wrong only at the 14th or 15th significant digit, or a probe or port that serves only that work. In those regions Excel itself is not correctly rounded (on a sample of 2,026 visibly-off cells, Excel was correctly rounded on 1%, with a median error of 48 to 330 ulp). Matching those digits means reproducing Excel's rounding noise. Decision 0267 accepts those cells. Reopen the task only if a cell it owns is first wrong at the 13th digit or earlier.
