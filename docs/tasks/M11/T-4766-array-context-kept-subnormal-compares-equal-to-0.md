---
id: T-4766
title: "Array-context kept subnormal compares equal to 0 in Excel (probe cells A3 and D1)"
milestone: M11
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Match Excel for a kept subnormal inside an array expression: `=(EXP({-745,-708}))=0` is TRUE in excel-graph, while the scalar `(EXP(-745))=0` is FALSE.

## Context
Found by T-4355 (probes/subnormal-saved-negative-and-array A3 and D1, excel-graph 2026-11-07). D1 is TRUE and spills nothing further; the engine gives FALSE. `INDEX(EXP({-745,-708}),1)*2^1000`-style recovery still works for a spilled cell (A5), so the flush may apply only in array-context comparison, or be a Graph artifact (decision 0042); the desktop question goes to M14 (decision 0039). Probe which before changing the engine.

## Acceptance criteria
- [ ] Probe cell D1 is fixed or recorded as DEVIATION / desktop-owned with evidence; numbers.md "Worksheet EXP (T-1931)" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
