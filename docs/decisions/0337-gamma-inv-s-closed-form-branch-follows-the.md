# 0337 — GAMMA.INV's closed-form branch follows the math: allow its R3 regressions

- **Status:** accepted (2026-11-07, T-3364). It applies decision 0188 R3's case 3 (owner ruling in 0188) and records
  the baseline regressions it implies, as T-3364's acceptance criteria say.
- **Context:**
  - Decision 0188 R3, case 3, and statistics.md "GAMMA.INV at a huge shape, closed-form branch" (`MATH-OVER-EXCEL`):
    where the gamma density at Excel's Wilson-Hilferty start underflows, Excel returns the start, about 1.000266
    times the true quantile, past G = 1e-4. There the engine follows the math. The branch serves GAMMA.INV,
    GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV.
  - The evidence is committed before the engine change, as R3 requires. The probe cases' 50-digit math sidecars
    (probes/t2982-gamma-inv-huge-shape, t3070-gamma-inv-band, t3350-chisq-inv-huge-df, incomplete-gamma-cap) hold
    every cell on the branch. Excel is 2.63e-4 to 2.66e-4 off the math on each of them, from shape 2.5E6
    (`CHISQ.INV.RT(1E-300,5E6)`) to 1E300. Gate A holds 132 points on the branch: 131 in the huge-shape batch
    e-T-3863, marked R3 by decision 0289, and accuracy/CHIINV/s1 A93 (T-4104, decision 0318).
  - Until now the engine returned Excel's start there from shape 1E9 (`START_SHAPE_MIN`), and the true quantile
    below it. R3's case 3 starts at shape 1E9. Below it (probes/t3350-chisq-inv-huge-df A205-A289,
    `CHISQ.INV.RT(1E-300,df)` for df 5E6 to 1E9) Excel is 2.63e-4 off the math too, but the owner ruled (2026-10-31,
    T-3409) not to widen R3 there: Excel's start is the target (T-4205, statistics.md "GAMMA.INV and CHISQ.INV at a
    huge shape").
- **Decision:**
  - Where the start underflows, `huge_shape_quantile` returns the true quantile: the unit-scale `gamma_quantile`
    times the scale. From shape 1E9 this is R3. Below it, the value is unchanged (it was already the true
    quantile), and T-4205 stays open to find Excel's start digits there. Refined at the scale itself, a quantile near f64::MAX
    (`GAMMAINV(0.5,1E300,1E8)`) was lost, and Excel's start there is a number. `START_SHAPE_MIN` is removed. The
    start and its fitted terms still pick the branches, so the `#NUM!` and true-quantile branches keep Excel's
    values.
  - The branch's probe cells from shape 1E9 are marked R3 in eval/accuracy/math-over-excel.tsv: 206 cells
    (t3070-gamma-inv-band 122, t2982-gamma-inv-huge-shape 64, t3350-chisq-inv-huge-df 19, incomplete-gamma-cap 1),
    each checked to have Excel more than 1e-4 off its committed math value and the engine within 1e-13 of it. The 8
    cells below 1E9 are not marked. Gate A's 132 T-3364 known
    failures leave eval/accuracy/known-failures.tsv, because they pass.
  - `rustsheet-eval update-baseline --allow-regressions` runs once in T-3364's PR. It records, and only records:
    - 59 of the branch's probe cells, and 3 of the gen-m9 cells below, as known failures against Excel (they now
      differ from Excel's value, by design);
    - five gen-m9 cells, multi-call formulas the math generator does not cover, so they carry no math value and
      no R3 mark. They are `GAMMAINV(1-1E-10,Data!C7,0.5)` (0a614cfe974b59c3 A327),
      `CHIINV(0.001,ROUND(9999999999999999,2)+…)` (4e55500bb8ad3cd8 A341), `CHIINV(1E-300,ROUND(9999999999999999,2))`
      (7bd219ec0c0c5e22 A239), `CHIINV(0.5,(Data!C$1+1E15)*RADIANS(1))` (a013a45a8fbcab4f A338) and
      `GAMMA.INV((2^Data!A2)^(…),AVERAGE(…),1E-300)` (d8266bc15533702a A296). Each has Excel / engine = 1.000266,
      the branch's signature. For `CHIINV(1E-300,1E16)` the normal bound df + z sqrt(2 df), with z = 37.05, gives
      1.0000005E16; ours is 1.0000005239E16 and Excel's 1.0002663E16. They raise five visible-parity lock lines by
      one each.
  - T-4205 stays open, outside R3, under the owner's T-3409 ruling.
- **Consequences:**
  - Gate A's known failures fall from 321 to 189, and its R3 points pass.
  - The probe cells count as R3 passes in the R1(c) rows, so the visible-parity lock of their cases does not move.
    The five gen-m9 cells count as visibly off until the math generator covers multi-call cells. They are listed in
    T-3364's Outcome.
  - Reviewers confirm that the baseline diff adds only these cells.
  - Rollback: revert T-3364's PR.
