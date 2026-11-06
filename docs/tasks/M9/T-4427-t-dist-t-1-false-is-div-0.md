---
id: T-4427
title: "T.DIST(t,1,FALSE) is #DIV/0! where pi(1 + t t) overflows"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T.DIST(t,1,FALSE) returns Excel's `#DIV/0!` where `π·fl(1 + t·t)` overflows but t·t does not, and the scratch
subnormal probes of T-2585 are committed as an eval case.

## Context
Found by T-2585's review (statistics.md "Student t density is Loader's dt", bullet "Subnormal results"). On excel-graph
(scratch recalc, 2026-11-06): `T.DIST(1E154,1,FALSE)` is `#DIV/0!` and `(T.DIST(1E154,1,FALSE))=0` is `#DIV/0!` too;
`T.DIST(3E153,1,FALSE)*1E300` is 3.5367765131532296E-8; `T.DIST(1E155,1,FALSE)` is `#NUM!` (t·t overflows, T-3554's
class). Ours is 0 at 1E154: `loader.rs` `excel_t_density` df 1 branch takes 1/inf. The same recalcs gave the
subnormal results the semantics bullet lists (df >= 3 flushed, df 2 kept), which have no committed case except
Gate A T.DIST/r4-T-2585 A138.

## Acceptance criteria
- [ ] A probes case (excel-probe skill) holds the df 1 overflow cells, the df >= 3 `x*1E300` and `(x)=0` cells and the
      df 2 kept-subnormal cells of the statistics.md bullet, filled on excel-graph.
- [ ] T.DIST(t,1,FALSE) is `#DIV/0!` where `π·fl(1 + t·t)` overflows and t·t is finite; find the exact edge between
      3E153 and 1E154 with the probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` where t·t overflows (T-3554).
