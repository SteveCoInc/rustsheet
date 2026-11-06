---
id: T-4467
title: "BESSELY order 4 next to its tiny-x overflow cut is 4 ulp below ours: find the rounding of the recurrence that reproduces probes/t4458-bessely-tiny-x-cuts Calc!D32"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BESSELY matches Excel's last digits next to its tiny-x overflow cut, starting with `BESSELY(2.036E-77,4)`, without
losing a cell that passes today.

## Context
- From T-4458 (probes/t4458-bessely-tiny-x-cuts, excel-graph, 2026-11-06; engineering.md "BESSELY at a tiny x and a
  small order"). The cut of every order 2 to 10 is the double model's. The one miss is Calc!D32:
  - `BESSELY(2.036E-77,4)` is -1.7783214437962194E+308 in both fills of the case;
  - ours, the double model `b * (2j * (1/x)) - a`, gives -1.7783214437962202E+308, 4 ulp away;
  - its neighbours 2.035E-77 and 2.037E-77 match, as do every other cell of the case and the 263 cells of
    probes/t4458-bessely-tiny-x-bisect.
- Reassociating the step does not reproduce it: `(2j/x)*b`, `(2j*b)/x` and exact steps all give
  -1.77832144379622E+308 or ours. Candidates to test, in order:
  - `y1`'s `1/x` and `ln x` terms kept in x87 extended precision;
  - the step's product kept extended before the subtraction;
  - an extended `inv_x`.
- That choice is a rounding model, so it is kind (c) of reviewer-excel-fidelity check 7 and owes decision 0188 R4's
  guards. The hold-out split must be fixed before scoring, and the fresh sweep must sit under eval/suites/probes.
  The miss is exactly 4 ulp, which is the edge of R4.4's web-vs-desktop reproducibility floor
  (docs/research/oracle-calibration.md). So first show that it reproduces, on a third fill and on desktop if one is
  available. If it varies, it is below the floor and should be recorded, not fitted.
- In the same fill, probe order 1 over the normal band [2.2250738585072014E-308, 3E-308), for example 2.5E-308,
  2.3E-308 and 2.2250738585072014E-308. T-4458 stopped at 3E-308, so "order 1 answers at every normal x" is open
  there.
- Owned cells: probes/t4458-bessely-tiny-x-cuts Calc!D32.

## Acceptance criteria
- [x] probes/t4458-bessely-tiny-x-cuts Calc!D32 passes, or engineering.md records why it is no target, with no other
      BESSELY cell changing verdict (probes/t4458-*, probes/m9-bessel-order-limits, probes-numeric/m9-bessel,
      ironcalc-calc BESSEL.xlsx, gen-m9).
- [x] If a rounding model is chosen by scoring, the Outcome shows the R4 guards: the hold-out, a fresh sweep, the Gate
      A result, the reproducibility floor, and no keep-matched branch.
- [x] `make check` green; reviewers approve.

## Out of scope
- BESSELY at a subnormal x (engineering.md: unreliable on excel-graph, no target).

## Outcome
- **Cause:** the miss was the literal, not BESSELY. Excel reads `=2.036E-77` (and `2036E-80`, `20.36E-78`) as
  2.0360000000000002E-77, one ulp above the nearest double (probes/t4467-bessely-d32-refill E1:E3). The double model at
  that x is Excel's -1.7783214437962194E+308. The miss reproduced in a third fill (D3), so it is above the
  reproducibility floor. None of the rounding candidates for `y1` or the step listed in Context reproduces it, and
  x + 1 ulp does.
- **Rule (numbers.md "Literals past n = 15 multiply by a table of 10^±16q"):** text with n = 16q + r, 16 or more places
  either way, reads as RN64(m × RN64(10^-r)), then × RN64(10^-16q) for q <= 16, or × RN64(10^-16(q-16)) and then
  × RN64(10^-256), stored to a double.
- **Engine:** new `x87_table_read.rs` with 64-bit tables of 10^±16q (q = 1..16), reached from `x87_text_to_f64`, so
  every `parse_excel_number` reader takes it. The places from -15 to 15 are unchanged.
- **R4 guards** (a structural rounding model chosen from ranked candidates, kind (c), so no `FITTED` tag):
  - Hold-out: probes/t4467-literal-large-exponent, Train seed 4467001 and Hold seed 4467002. The split was fixed in
    the case before any scoring.
  - Candidates scored on Train only (200 cells): nearest 90; m × RN64(10^-n) 140; m / RN64(10^n) 128; binary and
    1..15 + 16·2^k tables at most 84 of the 100 literals; 10^-16q with the 10^-256 cap first 3 misses. The model
    above scored 200/200 and was frozen (`frozen.py` in the session scratchpad).
  - Hold: 112/112 (nearest 62).
  - Fresh sweep after the freeze: probes/t4467-literal-large-exponent-fresh, seed 4467003, 85 literals. 170/170
    (nearest 20).
  - Gate A: these are literal reads, not Gate A functions, so the cases have no math sidecar (the generator writes
    none). The BESSELY refill has 13 math cells.
  - Reproducibility floor: D32 is the same in three fills, and every literal agrees between its literal and VALUE
    forms.
  - No keep-matched branch: the read applies at every place of 16 or more.
- **Also probed (engineering.md):** order 1 answers over the normal band down to 2.2250738585073E-308.
  `BESSELY(2.2250738585072014E-308,1)` is `#NUM!`, because the 15-digit cut is below DBL_MIN.
- **Eval delta:** probes 1 known failure fixed (t4458-bessely-tiny-x-cuts Calc!D32), 0 regressions. The 3 new cases (544 cells)
  all pass. No other suite moves: lifting the n <= 15 cap changes no other oracle cell.
