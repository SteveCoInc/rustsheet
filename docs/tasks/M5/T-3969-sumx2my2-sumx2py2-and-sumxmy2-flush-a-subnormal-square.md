---
id: T-3969
title: "SUMX2MY2, SUMX2PY2 and SUMXMY2 flush a subnormal square to 0 (probes/t3677 A12:A15)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMX2MY2, SUMX2PY2 and SUMXMY2 drop a square (x^2, y^2, (x-y)^2) that is below the smallest normal double, as SUMSQ
does, so probes/t3677-sumx-running-overflow-snap Calc!A12, A13, A14 and A15 pass.

## Context
- Probed by T-3677 (excel-graph, 2026-11-03): with M1:M2 = 1E-155, 1E-150 and N1:N2 = 0, 0, `SUMX2PY2(M1:M2,N1:N2)`,
  `SUMX2MY2(M1:M2,N1:N2)` and `SUMXMY2(M1:M2,N1:N2)` are exactly 1E-300 and `SUMX2MY2(N1:N2,M1:M2)` is -1E-300: the
  1E-310 square is dropped. The engine keeps it (1.0000000001E-300).
- numbers.md, "Subnormal results become 0" (the SUMX entry added by T-3677) and the SUMSQ rule (T-1057). The Gate A
  definitions in `tools/src/rustsheet_tools/accuracy/defs/sumx.py` model the same flush.
- Code: `sumx_fold` in `crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs`; reuse
  `arithmetic::flush_subnormal` (as SUMSQ does), not a new helper.

## Acceptance criteria
- [ ] probes/t3677-sumx-running-overflow-snap Calc!A12:A15 pass; `make eval-baseline` records them.
- [ ] No other probes or probes-numeric cell of the three functions regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
