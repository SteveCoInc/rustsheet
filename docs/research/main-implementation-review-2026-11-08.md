# Review: how main implemented the 2026-11-07 Excel-parity findings (2026-11-08)

Status: research hand-off, **docs only**. This compares main `89ee35e0d` with the prototype the earlier findings came from
(on base `2e192ce48`, Gate B 500/506) for each function family in `excel-parity-findings-2026-11-07*.md`. It also lists the
places where the T-4974 Excel probes show the earlier hand-off was wrong.

**Evidence used.**
- Main's commits and task outcomes (T-numbers and PR numbers are cited inline).
- Known-failure (`F`) lines in `eval/baseline/*.txt`, attributed to a function when the cell's formula calls it.
  "Shared" counts only cases that already existed at `2e192ce48`. The prototype's own baselines were never regenerated,
  so its counts come from a full eval report instead.
- Gate B per function: for the prototype, from the run above; for main, only the `e0abdcb2b` run is available (469/506).
- The 24 `probes/t4974-*` cases, scored on both trees (table below). On main, "fail" means F lines and "vis" means the
  `visible-parity.txt` lock. On the prototype, both come from the eval report.

| t4974 case | cells | prototype fail / vis | main fail / vis | main exact, prototype off |
|---|---|---|---|---|
| beta-f-inv-endpoint-saturation | 280 | 47 / 26 | 248 / 191 | 10 (A6:A8, F6:F8, J6:J8) |
| beta-inv-as109-accuracy-exponent | 624 | 302 / 12 | 410 / 8 | 34 (last digit) |
| beta-inv-dbl-min-residual | 132 | 112 / 111 | 120 / 120 | 0 |
| beta-inv-large-shape-round-trip | 861 | 190 / 24 | 228 / 38 | 24 (E6, E12, F1, F6, F12, F38, G6, G12 visible) |
| beta-inv-tiny-shape-num-boundary | 272 | 30 / 30 | 68 / 67 | 0 |
| binom-loader-store-chain | 234 | 169 / 10 | 189 / 13 | 0 |
| chisq-inv-rt-q-to-p-switch | 279 | 186 / 3 | 227 / 32 | 1 |
| chisq-inv-unit-half-boundary | 375 | 155 / 10 | 218 / 33 | 10 |
| f-inv-as109-tolerance-cap | 175 | 36 / 2 | 155 / 22 | 6 (J3, J13 visible) |
| f-inv-back-transform | 384 | 96 / 25 | 222 / 50 | 19 (B7, C6, C7, D6, W6, X6, AD7, AE7 visible) |
| f-inv-tiny-tail-neighbourhood | 90 | 11 / 11 | 90 / 64 | 0 |
| gamma-far-upper-tail-shapes | 110 | 89 / 25 | 89 / 25 | 0 (same code on both) |
| gamma-small-x-prefactor-boundary | 360 | 197 / 69 | 235 / 135 | 4 |
| gamma-split-complement-region | 255 | 50 / 0 | 84 / 0 | 3 |
| irr-negative-discount-and-caps | 63 | 0 / 0 | 25 / 17 | 0 |
| lognorm-density-holdout | 148 | 0 / 0 | 124 / 17 | 0 |
| moments-x87-adversarial | 204 | 0 / 0 | 0 / 0 | 0 |
| norm-s-inv-central-fingerprint | 226 | 140 / 0 | 144 / 0 | 0 |
| t-dist-df1-and-large-df-centre | 76 | 29 / 0 | 31 / 9 | 7 |
| t-dist-df1000-central-endpoint | 198 | 115 / 1 | 93 / 0 | 31 (all at df 1000 or 1001) |
| t-inv-joint-abs-rel-stop | 110 | 16 / 10 | 34 / 28 | 0 |
| t-inv-quarter-stop | 112 | 6 / 6 | 22 / 22 | 1 (C12) |
| t-inv-small-beta-tails | 60 | 15 / 1 | 33 / 16 | 3 (H5 visible) |
| t-inv-tiny-centre-region | 147 | 16 / 1 | 130 / 8 | 0 |
| **total** | 5,775 | **2,007 / 377** | **3,219 / 915** | 153 |

Read the last column as "cells where main is right and the prototype is not". The opposite direction (main fails, the
prototype is bit-exact) has 1,416 cells.

## Summary

| Family | Main status | Verdict | Action |
|---|---|---|---|
| Merged-cell import (findings 1) | T-4722 #4031, T-5044 #4046, T-5061 #4059 | (a) import matches; paste/autofill (c) | Land T-5045/T-5047; lower two stale visible locks |
| SKEW, SKEW.P, KURT (2) | T-4328 #4053 | (a), and main goes further | None; main's version is the reference |
| POISSON far upper tail (4) | T-3469 #3905 | (a), identical mechanism | Probe follow-up on other shapes (below) |
| BINOM.DIST.RANGE Loader stores (3) | not implemented (T-2146 #3370 is the earlier kernel) | (c) | Implement findings 3 |
| Gamma small-x prefactor (5) | not implemented (T-3219 todo) | (c) | Implement, with a corrected region |
| CHIINV / CHISQ.INV(.RT) (6) | not implemented (T-3146 blocked) | (c) | Implement; leave the dispatch cut bracketed |
| T.TEST / TTEST (7) | not implemented (T-3730 todo) | (c) | Implement findings 7 |
| T.INV small-beta tail (8) | not implemented (T-3363 todo) | (c) | Implement, with df 2 excluded |
| T.DIST.2T (9) | not implemented (T-2930 todo) | (c) | Implement with `n > SERIES_MAX_DF` |
| LOGNORM.DIST density (10) | not implemented (T-4321 todo) | (c); probe confirms 148/148 | Implement findings 10 as written |
| BETA.INV / BETAINV (addendum A) | T-3351 #3911, T-4958 #4040, T-4866 #4058, T-4865 #4072 | (b) mixed | Keep main's crossing and end rules; add AS 109 iterates only where evidenced |
| F.INV / F.INV.RT / FINV (addendum C) | T-1871 #3953 | (b); the prototype is closer overall | Implement C on top of T-1871 and T-3351 |
| F.TEST / FTEST (addendum D) | T-4882 #4054 (a different defect) | (c) | Implement D; keep T-4882 |
| T.INV.2T / TINV / CONFIDENCE.T (addendum B) | not implemented | (c) | Implement B; joint stop is incomplete |
| NORMINV / ERFC (addendum 2) | not implemented | (c) | Implement; low priority |
| IRR / NPV (findings 11, addendum 3) | research only: T-4835 #4034, T-4836 #4045, T-4797 #4063, T-4837 #4067 | (c) | Score addendum H on T-4836's bank before choosing |
| LINEST/LOGEST (12), CUMIPMT (13) | T-2690 todo; T-3731 time-boxed after two attempts | (c) | Implement findings 12 and 13 |

Gate B, prototype against main `e0abdcb2b`: BETA.INV 73/0 vs 73/2, GAMMA.DIST 108/0 vs 108/2, IRR 302/0 vs 302/5,
KURT 295/0 vs 295/3 (main has since fixed KURT), T.TEST 169/0 vs 169/3.

## Families that need action

### Merged cells
`rustsheet-xlsx/src/import/merged_cells.rs` and `import/worksheets.rs` on main now keep covered content. The remaining
diff with the prototype is comments only. `fixtures/merged-covered-values.xlsx` and `merged-covered-formula.xlsx` have
no F lines on main, but `visible-parity.txt` still allows 10 and 5 visibly-off cells for them. Run
`--lower-visible-lock` so a regression there is caught. Paste (`recreate_pasted_merge`, `covered_cells_clear_diffs` with
the pasted block) and autofill (`fill_covered_cell`) are still open as T-5045 and T-5047. Findings item 1 describes them,
with undo/redo tests.

### SKEW, SKEW.P, KURT
Main's `count_and_average.rs::standardized_moment` is the same kernel. Main also forms the outer factors
(`n/((n-1)(n-2))`, KURT's two terms) with `extended_*`, where the prototype used plain doubles. Both trees are 204/204 on
the adversarial probe. KURT's shared F lines went from 52 to 2 and SKEW's from 7 to 1. No action.

### POISSON and the far upper tail of Q
`incomplete_gamma/excel_tail.rs::excel_far_upper` is identical on both trees. The probe
`t4974-gamma-far-upper-tail-shapes` is visibly off in the same 25 cells on both. Examples:
- `CHISQ.DIST.RT(36,3)`: Excel 7.488376948795492E-08, engine …482;
- `CHISQ.DIST.RT(600,30)` = `POISSON.DIST(14,300,TRUE)`: Excel 2.962279801331723E-107, engine …176.

So the direct-deviance form is right where it was measured (shapes 5 to 10) but not at shape 1.5 or 15. Recommendation:
keep `FAR_TAIL_MAX_SHAPE` and the multiples-of-1/2 domain as they are, and add the 25 cells to T-3640/T-4954. Do not
widen the region on the strength of findings item 4.

### BETA.INV / BETAINV
Main's mechanism (`beta.rs::beta_quantile`) is the crossing of BETA.DIST's own CDF, plus point rules:
- `upper_quantile_near_one` (T-3351): 1 where the p > 1/2 quantile rounds to 1;
- `SHAPE_ONE_QUANTILE_CAP` 0.999999 (T-4958);
- direct `bgrat` where Cephes' tails are exactly 0 and 1 (T-4866);
- the `#NUM!` cuts of T-3862 and T-4713.

The prototype (`inverse_beta.rs::inverse_beta_search`, `BetaSearch::Excel`) returns AS 109 Newton iterates.

Across the five BETA.INV probe cases the prototype fails 681 cells (203 visible) against main's 1,074 (424 locked).
Main is still right in two places where the prototype is not:
- Near 1: `BETA.INV(1-1E-12,0.5,0.5)`, `(1-1E-12,10,0.1)` and `(1-1E-12,50,0.5)` are 1 in Excel and on main, and `#NUM!` in the
  prototype. The same rule decides f-inv-back-transform B7 and AD7, and their `#DIV/0!` dependants C7 and AE7.
- Large-shape round trips: E6, E12, F1, F38 and others. Excel returns x exactly; the prototype's early-iterate branch
  (`q.abs() <= EPSILON*min(p,1-p)` with `refine_increasing` or `nearest_crossing_from`) moves it by 1 to 17 ulp.

Recommendation: keep main's crossing as the default and keep all of main's end rules. Add the AS 109 iterate only behind
evidence that separates it from the crossing (ironcalc BETADIST_BETAINV H5/H26, gen-m9/4e55500bb8ad3cd8 A360, the
as109-accuracy-exponent rows where the prototype is exact and main is not, 142 cells). Each addition must leave the
"main exact" cells above unchanged. Gate B is 73/0 for the prototype against 73/2 for main at `e0abdcb2b`; main's shared F
lines fell from 100 to 48 with T-3351. The DBL_MIN, tiny-shape and endpoint groups are wrong on both trees (see the last
section). They stay with T-4994 and T-4865.

### F.INV / F.INV.RT / FINV
Main's T-1871 (`fisher.rs::f_quantile`, `estimate_solves`, `F_QUANTILE_ESTIMATE_RESIDUAL_MAX`, brcomp's far branch in
`incomplete_beta.rs`) keeps statrs' start where it solves the target and otherwise takes the crossing. The prototype runs
`inverse_beta::f_inverse_beta_tail` (the `ExcelF` profile) and then the compensated four-log back-transform
`EXP(LN(df2)+LN(x)−LN(df1)−log1p(−x))`. Across the three F cases the prototype fails 143 cells (38 visible) against main's
467 (136 locked). Main is exact where the prototype is off in:
- J3/J13: `F.INV(0.4,11,11)`, Excel 0.8552523165801218, prototype …224;
- the T-3351 cells above;
- C6/D6 and W6/X6: one beta ulp near 1, amplified by `1/(1-x)`.

Recommendation: implement addendum C on main, with three requirements. Keep T-1871's checked start as
`f_quantile_fallback`'s guard; the prototype dropped `estimate_solves` and its deep-tail test, so restore both. Route p > 1/2
through T-3351. Keep the R3 math-over-Excel check. F.INV Gate B is 227/1 on the prototype; its visibly-off cells (for example
gen-m9/589f42e67f4b4b8d A319 and 85f48d5cdfb24555 A12) are known failures on main as well.

### F.TEST / FTEST
Main's T-4882 makes an omitted sample `#VALUE!`, which is a separate defect. Addendum D's paired normalizer
(`bratio/ln_beta.rs::f_test_ln_beta`, used by `f_test_tail.rs`) is not on main: shared F lines are 136 and 55 on both
trees. Implement D, and keep T-4882's `CalcResult::EmptyArg` check. The prototype predates that check, so diffing it onto
main would delete it.

### Student-t family (findings 7, 8, 9; addendum B)
None of these is on main (`student_t.rs` and `t_dist.rs` changed only for lint scopes).
- **T.DIST.2T (item 9).** The prototype has `if t < 1.0 && n >= SERIES_MAX_DF` in `student_t.rs`. That is the off-by-one
  item 9 warns against. It costs 31 last-digit cells of t4974-t-dist-df1000-central-endpoint, 28 of them at df 1000,
  including the named control `T.DIST.RT(0.5,1000)` (Excel 0.30859254041693746, prototype …374). Main is exact there. Use
  `n > SERIES_MAX_DF` as item 9 says, and add df 999/1000/1001 controls.
- **T.INV small-beta tail (item 8).** Implement it, but outside df 2. `T.INV(1E-140,2)` is -7.071067811865557E+69 in Excel and
  on main; the prototype returns the closed form …476.
- **Joint stop (addendum B).** The prototype is better (joint 16/10 against 34/28; tiny centre 16/1 against 130/8), but
  see the last section for the cells it still misses.
- **T.TEST (item 7).** No probe; implement as written (169/0 against 169/3).

### Gamma and chi-squared (findings 5, 6)
Neither is on main. Both make Gate B functions pass (GAMMA.DIST, CHISQ.DIST(.RT), CHIINV, CHISQ.INV(.RT)), and the probes
favour the prototype in every gamma case except the far tail above. Implement them with the region corrections in the
last section.

### LOGNORM.DIST, BINOM.DIST.RANGE, NORMINV/ERFC
- LOGNORM.DIST: the prototype is 148/148 on the holdout, against 124 fails and 17 visible for main. Implement item 10 as
  written.
- BINOM.DIST.RANGE: `loader.rs::LoaderExponent` and `Bd0Branch::store` are not on main (shared F lines 138). The probe
  favours the prototype (169/10 against 189/13), with no cell where main is exact and the prototype is not.
- NORMINV/ERFC: not on main. The NORM.S.INV fingerprint has 0 visible cells on both trees. NORM.INV is still 102/4 on
  the prototype, so expect partial gains only.

### IRR / NPV
Main has research only, no kernel change (`financial_util.rs` is unchanged for IRR). T-4836's lead is "RATE's driver
plus the Newton fallback": 2,046 typed and 1,942 bit-exact of 2,555 cells. On t4974-irr the prototype with addendum H is
63/63 against main's 25 fails (17 visible), including `IRR({0.3,-2.5,1.005})` and the `#NUM!` at `IRR({-500,1,1,1,1,1})`.
Before choosing a mechanism, score addendum H's kernel on T-4836's `bank.py` cells and on T-4837's disagreement inputs.
The two models make different predictions: Newton against a d = −0.1 secant retry. A tie should be broken by the
negation-sensitive cells (t2927-irr-fallback-sweep A80, ironcalc IRR.xlsx General!L17).

### Re-implementation hazards
The prototype predates T-4856's per-file `#![expect(...)]` lint-debt headers, T-4882 and the T-1871 test.
`git diff main <prototype>` shows all three as deletions. Re-implement the changes on main; do not apply that diff.

## Where the probes show the earlier hand-off was wrong

1. **BETA.INV at p near DBL_MIN (addendum A, contract 1).** The hand-off called the converged root, without a flush, the
   target. Excel returns neither the converged root nor the 1E-6 seed:
   - The forward CDF of Excel's result is 1.12× p at 1E-307 (K2 1.1220898388726922E-307) and 1.76× p at DBL_MIN
     (K6 3.9074971845766525E-308).
   - A6, A7 and A9 (p from 2.22507385850721E-308 to 2^-1022·(1+2^-52)) agree to 14 digits, 3.6855664841263E-20.
   - The result is not monotone in p (A5 at 2.5E-308 is above A4 at 3E-308).

   The prototype converges (K2 = 9.9999999999995E-308) and is visibly off in 111/132 cells; main in 120. Corrected
   guidance: an early stop that neither tree reproduces. Do not use accuracy/BETA.INV/s1 A111 as evidence for running to
   convergence.
2. **Large-shape early-iterate trigger (contract 4 and the floating-point-resolution branch).** The prototype fires in
   rows 1, 6, 12, 19, 36, 38, 90, 122 and 123, where Excel gives x back. Excel's own early iterates are in rows 33, 34, 52,
   70, 71, 73, 86, 88, 105 and 106. The trigger is not the basym/ulp test the hand-off described. Default to the crossing
   (main) until T-4994 finds a rule.
3. **Tiny-shape `#NUM!` boundary (contract 2).** The contraction-count separator (11..597 for numbers, 672..1667 for errors)
   is falsified:
   - `BETA.INV(0.1,1,1E-3)`, `BETA.INV(0.5,1,1E-3)` and `BETA.INV(0.001,1,1E-6)` are 0.9999999999999999 in Excel, and
     `#NUM!` in the prototype;
   - `BETA.INV(0.9,1E-3,1)` is 1.1102230246251565E-16 (2^-53) in Excel and 1.7E-46 in the prototype.

   Excel ends one ulp inside the endpoint where the prototype errors or underflows. T-4865 finds that BETA.DIST at the
   iterates does not decide it either. Treat the boundary as unknown.
4. **Endpoint saturation.** At a first shape of 0.5, Excel's tiny-p BETA.INV does not depend on p:
   - `BETA.INV(1E-200,0.5,0.5)` = `BETA.INV(1E-300,0.5,0.5)` = 2.7055792171147168E-151;
   - at b = 50, both are 2.7053113740149512E-151.

   The prototype is `#NUM!` at all four. At b = 5E9, `BETA.INV(1E-300,2,b)` = 1.6790814881946113E-158 is about 60 times
   the power-law value the prototype returns. Addendum C describes the half-half seed only in the F caller. The probe shows
   that a saturation of this kind also governs the direct BETA.INV, so it belongs in the shared search, not in `ExcelF`
   alone. Near 1, adopt main's T-3351 rule.
5. **Gamma small-x prefactor region (item 5).** The `y = x/β <= 1` cut is wrong. In t4974-gamma-small-x-prefactor-boundary,
   every y up to and including 1 is within 4 ulp. Every y above 1 is visibly off: 1+2^-52, 1+4·2^-52 and 1.001, at shapes 8
   and 30 and β 1 and 2 (`GAMMA.DIST(1.001,30,1,TRUE)`: Excel 1.4752762130359831E-33, prototype …978). So Excel keeps
   the separately rounded factors, or another non-accurate form, past y = 1. Probe y in (1.001, 2] before choosing the cut.
6. **Gamma far upper tail (item 4).** See the POISSON subsection: shapes 1.5 and 15 are visibly off on both trees.
   Item 4's form should not be generalized.
7. **F back-transform and tiny tail (addendum C).**
   - `F.INV(1E-300,11,3)` and `F.INV(1E-250,11,3)` are `#NUM!` in Excel and numbers on both trees.
   - Where BETA.INV is 1, `x/(1-x)` is `#DIV/0!` in Excel, not `#NUM!`.
   - Seven deep-tail cells at df1 7 to 11 and p 1E-250 to 1E-300 (A11, A12, B7, B8, C7, C9, E8) differ in the 13th to
     14th digit.

   The 758 → 859/995 replay gain stands. The claim that one cell is left (226/227) does not extend to these probes.
8. **Student-t joint stop (addendum B).** The listed controls hold. Still visibly off in the prototype:
   - `T.INV.2T(0.5,16)`, `(0.5,23)` and `(0.5,25)`;
   - `(0.4,4)`, `(0.4,11)` and `(0.6,7)`;
   - `(0.8,5)`/`(1.2,5)` and `(0.8,9)`/`(1.2,9)`.

   Add: `T.INV(0.255,8)` (quarter stop C12) and `T.INV(1E-140,2)` (item 8's domain; exclude df 2). The joint stop is
   necessary but not sufficient. Leave T-4998's four cells to it, and do not claim the family closed.
9. **IRR caps (item 11).** At the time of PR #4002, item 11 alone left 4 of the probe's cells visibly off (T-4974:
   51/8/4). Addendum H closes all 63. Item 11 is superseded: implement H's domain rule (rule 6) together with the
   uniform secant, not item 11 on its own.
10. **Loader store chain (item 3).** The pmf rule holds. Ten multi-point BINOM.DIST.RANGE cells at n = 50 and 100 are still
    visibly off (B54, B57, B62, B63, B65 to B70). Excel's values for mirrored ranges differ:
    - `BINOM.DIST.RANGE(50,0.4,5)` is …013 and `(50,0.6,45)` is …0124;
    - the prototype gives …005 for both.

    The range summation is orientation-dependent in Excel and not in the prototype. Item 3 covers the pmf only.
11. **CHISQ.INV around unit x = 0.5 and the Q-to-P cut (item 6).**
    - Unit x: the prototype still misses `CHISQ.INV.RT` at df 5, x 0.82 to 0.94 (unit x 0.41 to 0.47), and at df 8, x 0.98
      to 1.10, which straddles 0.5.
    - Cut: T-4974 measured that RT(p) equals INV(1-p) from 0.9999 + 2E-6 to 0.99999, and that it differs at 0.9999. The
      prototype misses A38:A41, within 4 ulp above 0.9999.

    So the switch is not exactly the dispatch `p <= 0.9999`. Keep it labelled bracketed, now (0.9999, 0.9999 + 2E-6], and
    let T-4996 probe the 4-ulp rows with comparators.
