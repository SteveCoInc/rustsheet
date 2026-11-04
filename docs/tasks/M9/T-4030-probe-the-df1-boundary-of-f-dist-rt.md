---
id: T-4030
title: "Probe the df1 boundary of F.DIST.RT #NUM! at df2 1E10 (df1 61 to 78) and its mechanism (T-4017 follow-up)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Find the df1 at which `F.DIST.RT(x,df1,1E10)` and `F.DIST(x,df1,1E10,TRUE)` turn to `#NUM!` (T-4017: numbers at df1 60, `#NUM!` at 79) and what decides it.

## Context
- docs/excel-semantics/statistics.md, "Incomplete beta at a huge first shape" (T-4017 sweep) and "F-distribution df limit (T-1530)", whose per-df cut does not explain it.
- Also probe df2 1E10 against 5E9 and 2E10 at df1 61 to 78, and BETA.DIST at the matching shapes (b = 39.5 to 5E9), to tell a df cut from a bratio branch.

## Acceptance criteria
- [ ] Oracle-filled probes-numeric cases (valid arity) bracket the boundary to one df1 and name the mechanism, recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
