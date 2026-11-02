---
id: T-3861
title: "CONVERT prefix last digits: target prefixes y..n and Y, and squared or cubed prefixes (probes/convert-unit-names)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
CONVERT with a prefixed unit matches Excel to the last bit: a target prefix of 1E-9 or smaller or of 1E24, and a prefix on a square or cubic unit, on either side.

## Context
T-3817 added probes/convert-unit-names (excel-graph). Its unit-string cells all match; 119 cells are one or
two ulps off (NumericUlp); these 116 come from the prefix scale, not from which strings parse:
- target prefixes Y, n, f, a, z, y on a linear unit (`CONVERT(1,"m","nm")` is 1E9 in Excel, ours
  999999999.9999999 = 1 × (1/1E-9)): Sheet1!B1153, B1163, B1165:B1168, B1175, B1185, B1187:B1190, B1197, B1207, B1209:B1212, B1219, B1229,
  B1231:B1234, B1241, B1251, B1253:B1256, B1263, B1273, B1275:B1278, B1285, B1295, B1297:B1300, B1307,
  B1317, B1319:B1322, B1329, B1339, B1341:B1344, B1351, B1361, B1363:B1366, B1417, B1427, B1429:B1432,
  B1439, B1449, B1451:B1454, B1461, B1471, B1473:B1476;
- squared and cubed prefixes on either side (`CONVERT(1,"dm2","m2")` is 0.01, ours 0.1 × 0.1 =
  0.010000000000000002; `"cm3"` to `"l"` is 0.001, ours 0.0010000000000000002; `"Pm3"` to `"l"` is
  9.999999999999999E47, ours 1E48): Sheet1!A1380, A1386:A1388, A1398, A1402:A1405, A1407:A1410, A1525, B684, B692, B764, B772, B1374,
  B1376:B1377, B1380, B1383:B1386, B1388, B1395:B1397, B1402:B1405, B1408:B1410, B1525.
A first hypothesis to test, not a finding: Excel keeps each prefix power (and its reciprocal) as the
correctly rounded decimal constant (1E-4 for d², 1E9 for 1/n) instead of computing it with multiplies
and a division. The order rule ("CONVERT constants and evaluation order" in engineering.md) and every
cell of ironcalc-calc CONVERT.xlsx and the convert-* probes must keep passing. Code: `parse_unit` and the
end of `convert_of` in crates/rustsheet-engine/src/functions/engineering/convert.rs. The model is a
structural choice: meet reviewer-excel-fidelity check 7 (hold-out and fresh sweep).
Not owned here: Sheet1!B33 (`m` to `ft`), B961, B969 (`m2` to `ly2`, `ly^2`) are non-base factor digits, T-2047.

## Acceptance criteria
- [ ] Every cell listed above passes, with the prefix-scale rule stated in engineering.md "CONVERT unit names" or
      "CONVERT constants and evaluation order" (VERIFIED, cells cited).
- [ ] No CONVERT cell of any suite changes from pass to fail.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-base factors (T-2046, T-2047) and which unit strings parse (T-3817).
