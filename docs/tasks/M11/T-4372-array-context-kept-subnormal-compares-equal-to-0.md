---
id: T-4372
title: "Array-context kept subnormal compares equal to 0 in Excel (probe cells A4 A5 A7 G4 G5 G7)"
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Match Excel for a kept subnormal inside an array / spill result: the engine keeps it (`=(EXP({-745,-708}))=0` is FALSE), excel-graph answers TRUE.

## Context
Found by T-4355 (probes/subnormal-saved-negative-and-array, excel-graph 2026-11-05). Cells A4, A5, A7 (value 0 vs engine 5e-324) and G4, G5, G7 (`(x)=0` TRUE vs FALSE) are visibly off. Scalar `(EXP(-745))=0` is FALSE (probes/subnormal-saved-as-zero), yet `INDEX(A4#,1,1)*2^1000` still recovers 2^-74, so the array element may be flushed only in the comparison, or Graph's array evaluation differs from the desktop's. Probe which (desktop question goes to M14, decision 0039) before changing the engine. Recorded in numbers.md "Worksheet EXP (T-1931)".

## Acceptance criteria
- [ ] The probe's off cells are fixed or recorded as DEVIATION/desktop-owned with evidence; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
