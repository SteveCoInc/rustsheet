---
id: T-4885
title: "F quantiles at a df of 1E5 or more: the engine's BETA.INV mapping is the math where Excel is 1E-13 to 1E-5 off it (T-4857 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
F.INV, F.INV.RT and FINV at a df of 1E5 or more give Excel's values on the T-4857 cells, where today the engine
returns the true quantile and Excel its own BETA.INV's.

## Context
- statistics.md "F quantile at a huge df goes through Excel's BETA.INV (T-1538)": Excel maps its BETA.INV to F, and
  that BETA.INV misses its own cdf's crossing; T-1538 matched the cells it had (mostly p = 0.5 and p = 1 ± 1E-12).
  `fisher.rs` `f_quantile_through_beta` follows it.
- T-4857's probe (probes-numeric/t4857-f-classes, excel-graph 2026-11-07) put the mapping on pairs and p no earlier
  cell had: where the 50-digit math is available the engine is within 1E-15 of it on most cells (1.1E-9 at A1007), and
  Excel is off it by 1E-13 to 1E-5; many cells have no math value (mpmath's budget at a huge shape). Examples:
  `F.INV.RT(0.05,1E10,5)` Excel 4.364995965811329, math and ours 4.3649967469460 (1.8E-7, A2063, and the same at df
  9999999999, 1E10 + 0.9 and 1E10 + 0.5: A2059, A2067, A2075); `FINV(0.5,1E10,21)` 5.4E-8 off (A1308);
  `F.INV.RT(0.5,24,5002015517)` 2.5E-8 (A1007); `FINV(DBL_MIN,1742447,2133203)` 1.4E-5 between Excel and ours
  (A1236). The t1538 cells still match, so Excel's stop depends on p and the shapes in a way T-1538's model does not
  cover.
- T-2555 (blocked) studies BETA.INV's early stop at a tiny quantile; T-1871 (in flight) touches F.INV.

Cells (probes-numeric/t4857-f-classes): A903, A921:A923, A1006:A1007, A1049, A1069, A1088, A1134, A1155, A1173:A1174, A1195, A1214:A1215, A1222, A1236, A1238, A1252, A1255:A1257, A1262, A1265, A1269, A1274:A1275, A1280, A1287, A1289:A1290, A1293:A1294, A1303:A1305, A1308, A1310, A1314, A1317:A1318, A1413, A1431, A1449, A1991, A2059, A2062:A2063, A2066:A2067, A2074:A2075, A2078, A2082, A2086, A2094, A2097:A2098, A2100:A2101, A2103:A2104, A2109:A2110, A2113, A2116, A2118:A2119, A2124:A2125

## Acceptance criteria
- [ ] A model of Excel's BETA.INV stop (or of the F mapping) that matches these cells within decision 0188 R4's
      guards, with t1538-f-inv-huge-df and t1538-f-quantile-switchover not regressing; or a time-boxed block with
      the measured residue.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Tiny-p cells where Excel is orders of magnitude off (T-4884).
