---
id: T-3861
title: "CONVERT prefix last digits: target prefixes y..n and Y, and squared or cubed prefixes (probes/convert-unit-names)"
milestone: M9
lane: functions-engineering
status: done
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
- [x] Every cell listed above passes, with the prefix-scale rule stated in engineering.md "CONVERT unit names" or
      "CONVERT constants and evaluation order" (VERIFIED, cells cited).
- [x] No CONVERT cell of any suite changes from pass to fail.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-base factors (T-2046, T-2047) and which unit strings parse (T-3817).

## Outcome
CONVERT applies its two prefixes as one factor: the nearest double to their combined power of ten (a
squared or cubic unit multiplies the prefix's exponent by 2 or 3), times the binary prefixes' exact ratio,
multiplied into the factor ratio last (`convert.rs` `parse_unit`, end of `convert_of`). This replaces the
prefix products and the `* 1/P_to` reciprocal, and the special case that combined the prefixes only when the
from-prefix overflowed: one factor keeps `CONVERT(1E290,"Ym","Ym")` at 1E+290 without it. Rule and evidence in
engineering.md "CONVERT constants and evaluation order".
- Model choice (decision 0188 R4, a structural model, kind (c): no fitted constant, so no `FITTED` tag). The
  task's hypothesis (each prefix and the target's reciprocal as its own nearest constant) was tried first; it
  fixed the listed cells but broke probes/convert-temperature-ov00 and ov01 (1E24 × 1E-24 is 1 - 2^-53). The
  combined power came from those two cells, before any population scoring. Then 18 forms were scored on the
  1,145 prefixed literal CONVERT cells with oracle values (probes/convert-*, convert-unit-names): combined
  power 1,137; nearest constants per prefix 1,103; the previous form 979; the other 15 between 961 and 1,027.
- Hold-out (R4.1). The split is 30% by SHA-256 of case and cell (cell hash below 3 mod 10). It was not fixed
  before the population was scored, so it is disclosed as after the fact. The model has no free parameter to
  tune, and the fresh sweep below is the out-of-sample test. No margin was stated in advance.

  | half | cells | combined power | correctly rounded | previous form |
  |---|---|---|---|---|
  | train | 807 | 801 | 742 | 688 |
  | hold-out | 338 | 336 | 314 | 291 |

  All 1,145 are within 4 ulp of Excel under the combined power. The 8 misses are 1 or 2 ulp.
- Fresh sweep (R4.2), generated after the model was frozen: new probes-numeric/t3861-convert-prefix-sweep,
  150 seeded calls (seed 3861; generator in the case description's terms: random class, unit pair and decimal
  or binary prefix on one or both sides, value 1 or log-uniform in 1E-12..1E12), excel-graph. Bit-exact 142
  (combined power), 123 (correctly rounded), 97 (previous form); 150 of 150 within 4 ulp and 15 digits.
  Gate A envelope: 150 of 150 inside (against the exact rational conversion of the engine's factors).
  Misses: A6, A56, A70, A85, A89, A106, A109 have a light-year or parsec factor (T-2047's digits; A109,
  `CONVERT(0.00041,"parsec","Pm")`, misses under every form), and A52 mixes a binary and a decimal prefix
  (T-3975). The previous form got A52 and A89; the sweep gains 45 net.
- Eval (`make eval-baseline`): probes known failures 4,631 to 4,515 (116 fixed, all in convert-unit-names;
  left there: B33, B961, B969, T-2047), 0 regressions in every gated suite; visible parity 0 in new cases.
  The new sweep case is in probes-numeric (ungated).
- Follow-up: T-3975 (mixed binary and decimal prefixes).

## Hold-out plan (round 2, stated before the batch exists)
Fidelity review round 1 found the hash split above contaminated: it was fixed after the population was scored.
This plan is committed before the new batch is generated, filled or scored.
- Hold-out: a new probes-numeric case t3861-convert-prefix-holdout, 150 calls from the sweep generator with seed
  38612 (the sweep used 3861), filled on excel-graph after this commit. Nothing is scored on it before the fill.
- Train: the 1,145 prefixed literal cells the model was chosen on (rate 801 + 336 = 1,137, 99.3% bit-exact).
- Margin, fixed now: the combined power passes if (1) its hold-out bit-exact count beats the correctly rounded
  conversion's on the same cells, and (2) its hold-out bit-exact rate is at least 91.3%, train's rate less 8 points.
  Why 8 points: the generator draws light-year and parsec units far more often than the train population does, and
  those factors miss under every form (T-2047). The post-freeze sweep lost 4.6 points that way, and 8 allows for
  that share varying between two draws of 150.
