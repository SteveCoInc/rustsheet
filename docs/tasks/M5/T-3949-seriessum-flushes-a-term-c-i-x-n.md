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
- `SERIESSUM(1E-150,2,0,{1E-10,23000})` (term-sum case A1) is exactly 2.3E-296 in Excel: the subnormal term 1E-310 is dropped before the sum, so the
  flush is per term; ours is 2.30000000000001E-296. Normal terms that cancel to a subnormal sum keep it (term-sum case A2), which the engine matches.
- A subnormal power (`SERIESSUM(1E-154,2,0,{1E-10})`, `(1E-160,2,0,{1,1E+300})`) is already flushed by the engine and matches.
Code path: `seriessum_of` (crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs).

Owned cells:
- probes/seriessum-subnormal-power-and-term.case.json (4): Sheet1!A2, B2, A9, B9
- probes/seriessum-subnormal-term-sum-and-power-points.case.json (1): Sheet1!A1

## Acceptance criteria
- [ ] probes/seriessum-subnormal-power-and-term Sheet1!A2, B2, A9, B9 and term-sum A1 pass, and no passing cell of any suite changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `^` operator (T-3950).
