# PROBE-REQUEST val4 — F.INV family (r8fin, 2026-11-10)

Three unfilled spec cases in `eval/suites/probes/cases/`, same `rustsheet-case/v1` layout as val3
(every formula cell is in `check`; no expected values; each workbook carries the `Precision` canary
sheet, whose B2:B4 must read 0.9999999999999998, 1, 1.0000000000000002 and C2:C4 -1, 0, 1).
Fill each as its own workbook with the excel-graph backend; read raw values, not displayed text.
Frozen predictions: `docs/research/val4-fin-predictions.json` (engine hypotheses, never oracle values).

Background (val3 scored, T-5107): the worksheet AS 109 trace of val3-fin-01 equals the private
`BETA.INV` bit for bit; the search stops where `CDF - p` is below 2^-1022; `F.INV` is the
double-double four-log mapping of that coordinate. With bit-exact forward kernels at shapes above 1
the whole val3 trace and all 17 adjacent-p returns are reproduced. What is left for the three
functions' Gate B cells is the endpoint-saturation class (a shape of 1/2), the stored ln B at a
larger shape above 8, and one flipped moderate-p search. These probes pin exactly those.

| Case | Checked formulas | What it pins |
|---|---:|---|
| `val4-fin-01` | 9,618 | Full worksheet trace of `BETA.INV(1E-200,0.5,0.5)` / `F.INV(1E-200,1,1)`: about 303 one-third steps from the 1E-6 end seed, Excel's cdf and density at shape 1/2 on every iterate |
| `val4-fin-02` | 900 | cdf and density ladders at a = 1/2 with b = 1/2, 1, 3/2, 9/2, 5 on x = 1E-6·3^-n; 25 saturated inverse returns in both orientations |
| `val4-fin-03` | 3,471 | Stored ln B of bpser for 181 shape pairs with the larger shape above 8 (two tiny x each, GAMMALN beside); the flipped trace behind `F.INV.RT(1E-16,9,3)` |

## val4-fin-01 — saturation trace at shape 1/2
`Seed` computes AS 109's small-shape first estimate in cells (B4:B9) and replaces an end estimate by
1E-6 (B10). `Trace` is val3-fin-01's trace with the same formulas (columns O, P, S, T dropped): 640
trial rows, a rejected trial and its one-third retry on separate rows. Stop codes as in val3
(0 continue, 1 zero residual, 2 unusable slope, 3 squared-step floor 1E-300, 4 unchanged proposal).
- **Expect:** Seed!B9 = 0 (the estimate underflows), B10 = 1E-6; 303 accepted rows; stop code 3
  near row 609; Seed!B13 = Seed!B11 (trace = private BETA.INV), B15 = 0. Cached Excel:
  `BETA.INV(1E-200,0.5,0.5)` = 2.7055792171147168E-151, `F.INV(1E-200,1,1)` = 2.705579217114786E-151.
- **If B13 = B11:** the public-kernel search is confirmed at shape 1/2; Trace!C:D then give 303
  cdf/density pairs to fit the shape-1/2 kernels bit for bit (the engine today is 1.7E-13 off the
  end point, i.e. its per-step ratio cdf/density is about one ulp off on average).
- **If B13 ≠ B11:** the seed rule, the retry rule or the floor differs; rows 2:9 (first two steps)
  and the stop row decide which. `INCOMPLETE` (no stop in 640 rows) is inconclusive, not a cap.

## val4-fin-02 — kernel ladders and saturated returns
`Ladder!B` is the checked raw x (so the argument is known exactly); C:L are cdf and density for the
five shapes. `Inverse` lists 25 returns: left-tail saturations at (1,1), (1,2), (1,3), (1,9), (1,10),
their BETA.INV coordinates, and the mirrored right-tail calls.
- **Expect:** each right-tail return equals the four-log mapping of the saturated flipped coordinate
  (`F.INV.RT(1E-300,3,1)` = 1.2320245435514617E+150 and `FINV(1E-200,9,1)` =
  4.1067731191132565E+149 are cached); `F.INV.RT(1E-300,2,1)` = `F.INV.RT(1E-200,2,1)` (p drops out
  of a saturated search).
- **Decides:** whether one kernel model reproduces cdf and density at a = 1/2 across b (closed
  arctangent form at b = 1/2, bpser with the split z elsewhere, the compensated small-shape
  density), which the 303-step searches need at every step.

## val4-fin-03 — stored ln B above 8, and a flipped search
Sheet `L`: for each (a, b) two x of the form odd·2^-k (raw, checked), cdf and density at both, and
GAMMALN(a), GAMMALN(b), GAMMALN(a+b).
- **Expect:** under the split-z rule (`z = fl(a·LN x) - L`, `zh = fl(z)`, `zl = z - zh`,
  `I = fl(fl(EXP(zh)·s)·fl(1 + zl))`) every cdf cell is reproduced by a **stored double** L within
  2 ulp of the correctly rounded ln B, the same L at both x of a row. No GAMMALN sum gives it for a
  larger shape above 8 (val3-fin-02 K2:K7 imply offsets 0, 0, +1, 0, 0, +1 ulp at shapes 16.5/6 …
  42.5/15), so the 181 implied values are the data for identifying Excel's form there.
- **Refutes the rule** if some row admits no double L, or different L at its two x.
Sheets `Seed`/`Trace`: val3-fin-01's trace on `I_s(1.5, 4.5) = 1E-16` (df entered swapped, 96 rows).
- **Expect:** Seed!B13 = Seed!B11; Seed!B12 = `F.INV.RT(1E-16,9,3)` = 60647458731.09812 (cached).
  The candidate engine agrees with its own trace but returns 60647458731.098335, so Excel's trace
  must leave the engine's at some row: a kernel bit (then L or the density at these shapes), the
  stop row, or the mapping of 1 - s over s (compare Seed!B14).

## Not in this batch
Huge-df cells (`F.INV(1E-200,1E+10,6)`, `FINV(1E-20,10000000000.5,30)`, `F.INV.RT(0.001,99999,1)`):
their searches run on bratio branches (bgrat, basym) whose kernels are not bit-exact yet; a trace
there is premature. The iteration cap / `#NUM!` class of val2-fin-03 is re-scored offline first.
