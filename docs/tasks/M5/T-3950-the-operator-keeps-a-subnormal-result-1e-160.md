---
id: T-3950
title: "The ^ operator keeps a subnormal result (1E-160^2 is not flushed; POWER is) (probes seriessum-subnormal-power-and-term)"
milestone: M5
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The ^ operator returns Excel's value where the result is subnormal: the subnormal is kept, not flushed to 0.

## Context
Found by T-3942's probe (probes/seriessum-subnormal-power-and-term, excel-graph, 2026-11-03; docs/excel-semantics/numbers.md
"SERIESSUM: a power or term below the smallest normal double is 0"):
- `=1E-160^2` keeps the subnormal 1E-320 in Excel: its `=0` companion B15 is FALSE and `=1E-160^2*1E+300` (A16) is
  9.99988867182683E-21, not 0. Ours flushes (B15 TRUE, A16 0).
- `POWER(1E-160,2)*1E+300` (A17) is 0: POWER flushes, `^` does not. numbers.md "Subnormal results become 0" lists `=2^-1074`
  as 0 on a Graph value that reads 0 for a kept subnormal too (decision 0042), so it does not conflict.
Code path: the `^` operator in crates/rustsheet-engine (arithmetic power).

Owned cells:
- probes/seriessum-subnormal-power-and-term.case.json (3): Sheet1!A16, B15, B16

## Acceptance criteria
- [ ] probes/seriessum-subnormal-power-and-term Sheet1!B15, A16, B16 pass, and no passing cell of any suite changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SERIESSUM's term (T-3949) and POWER, which already flush.
