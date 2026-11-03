---
id: T-3949
title: "SERIESSUM flushes a term c_i·x^(n+i·m) that is subnormal to 0 (probes seriessum-subnormal-power-and-term)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SERIESSUM gives Excel's value where a term c_i·x^(n+i·m) is subnormal: 0, not the subnormal.

## Context
Found by T-3942's probe (probes/seriessum-subnormal-power-and-term, excel-graph, 2026-11-03; docs/excel-semantics/numbers.md
"SERIESSUM: a power or term below the smallest normal double is 0"):
- `SERIESSUM(1E-150,2,0,{1E-10})` (power 1E-300 is normal, the term 1E-310 is subnormal) is a true 0 in Excel (A2, `=0` companion B2 TRUE);
  ours is 1E-310 (B2 FALSE). The same with a negative x (A9, B9).
- A subnormal power (`SERIESSUM(1E-154,2,0,{1E-10})`, `(1E-160,2,0,{1,1E+300})`) is already flushed by the engine and matches.
Code path: `seriessum_of` (crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs).

Owned cells:
- probes/seriessum-subnormal-power-and-term.case.json (4): Sheet1!A2, B2, A9, B9

## Acceptance criteria
- [ ] probes/seriessum-subnormal-power-and-term Sheet1!A2, B2, A9, B9 pass, and no passing cell of any suite changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `^` operator (T-3950).
