---
id: T-4994
title: "BETA.INV probe misses from t4974: DBL_MIN residual, tiny-shape #NUM! boundary, endpoint saturation, large-shape round trip"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
BETA.INV (and BETAINV, F.INV through AS 109) matches Excel on the visibly-off cells of the t4974 BETA.INV probes (PR #4002 groups G07, G21, G22, G23).

## Context
Found by T-4974, which filled the probe suggestions of PR #4002 (docs/research/probe-suggestions-2026-11-07, merged to main as 8001d80b1)
with excel-graph. The predictions there came from a prototype of the 2026-11-07 Excel-parity fixes that is not on main;
compare against them before designing a fix.
- G22 (DBL_MIN residual) is falsified: Excel's quantile continues smoothly through p = DBL_MIN (A6 `BETA.INV(2.22507385850721E-308,..)`
  Excel 3.6855664841262965E-20, the prototype 2.2394449712624932E-20; K6, the forward CDF of it, Excel 3.9E-308, prototype 0),
  so Excel does not flush the subnormal first residual.
- G23 (large-shape round trip): Excel returns an early iterate (F != 0) in rows 33, 34, 52, 70, 71, 73, 86, 88, 105 and 106; the
  prototype's rule fires in rows 1, 6, 12, 19, 36, 38, 90, 122 and 123 where Excel gives x back (including both audit rows).
- G21 and G07: the number-vs-`#NUM!` boundary and the endpoint saturation values differ in the cells below.

Owned cells (247, visibly off against Excel):
- probes/t4974-beta-inv-dbl-min-residual.case.json (97): P!A2:A7, A9:A11, B2:B7, B9:B11, C1:C7, C9:C11, D2:D7, D9:D11, E6:E7, E9, F1:F7, F9:F11, G2:G7, G9:G11, H1:H7, H9:H11, I2:I7, I9:I11, J1:J7, J9:J11, L2:L7, L9:L11
- probes/t4974-beta-inv-tiny-shape-num-boundary.case.json (64): P!A2:A3, A9, B1:B3, B9:B10, C1:C2, C9:C10, D1, D10, E3, F3, G3, H2, H9, I2:I3, I9, J1:J3, J9:J10, K1:K2, K9:K10, L1, L10, M3, N3, O3, P2, P9, Q2:Q3, Q9, R1:R3, R9:R10, S1:S2, S9:S10, T1, T10, U3, V3, W3, X2, X9, Y6, Z2:Z3, Z10:Z11, Z18:Z19
- probes/t4974-beta-f-inv-endpoint-saturation.case.json (48): P!A4:A8, F6:F8, I4:I5, J6:J8, K5, L1, L4:L5, M4:M5, M7:M8, N1, N4:N5, O7:O8, P1, P4:P5, Q7:Q8, R1, R4:R5, S7:S8, T1, T4:T5, U7:U8, V1, V4:V5, W7:W8, Y4:Y5
- probes/t4974-beta-inv-large-shape-round-trip.case.json (38): P!E2, E19, E33, E36:E37, E54:E55, E69, E71, E73, E86, E105, F19, F33:F34, F36, F52, F54, F69:F71, F73, F86, F88, F105:F106, G2, G19, G33, G36:G37, G54:G55, G69, G71, G73, G86, G105
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/t4994-shape-one-fresh (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] The owned cells are within half a unit of Excel's 15th significant digit (or the same error) and leave
      eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] docs/excel-semantics/statistics.md (financial.md for IRR) records the rule the probe settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit cells of the same cases.
