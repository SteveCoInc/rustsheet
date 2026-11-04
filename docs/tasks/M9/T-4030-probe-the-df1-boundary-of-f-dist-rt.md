---
id: T-4030
title: "Probe the df1 boundary of F.DIST.RT #NUM! at df2 1E10 (df1 61 to 78) and its mechanism (T-4017 follow-up)"
milestone: M9
lane: research
status: done
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

## Outcome
- Probe `probes-numeric/t4030-f-dist-rt-df1-boundary` (235 cells, excel-graph): at df2 1E10 the `#NUM!` band starts at
  df1 77 (b = 38.5), df1 80 and 100 are numbers; at df2 5E9 it starts at df1 79; the same edges hold for
  `BETA.DIST(x,df2/2,df1/2,TRUE)`, so the mechanism is a bratio branch on (a, b), not a df cut. Edge as a function of a
  tabulated (1E9 and 2E9: none to b = 39.5; 2.5E9 to 3E9: 39.5; 4E9 and 5E9: 38.5; 6E9: 37.5) in statistics.md.
- Not implemented (research lane). Follow-up: the engine's F/beta path needs this band (file via planner/gap-triager).
- Eval delta: none (new probe cases, no engine change).
