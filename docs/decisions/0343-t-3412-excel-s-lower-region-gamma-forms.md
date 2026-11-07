# 0343 — T-3412: Excel's lower-region gamma forms, four baselined cells one ulp off

- **Status:** accepted (2026-11-07, T-3412). It records an owner ruling under the standing rule that a regression
  outside decision 0083's allowance needs its own record; it adds no rule.
- **Context:** T-3412 gives GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST and the POISSON cdf Excel's own
  forms in the lower region (`incomplete_gamma::excel_lower_above_shape_one`, `excel_lower_region_upper`,
  `excel_lower_density`; docs/excel-semantics/statistics.md, "Lower region at a shape above 1 (T-3412)"):
  - the cdf at a shape above 1 and x/scale up to 1 is the leading-factor series form
    `EXP(a*LN(x))/EXP(GAMMALN(a+1))*EXP(-x)*series` (T-3060's form below shape 1);
  - the upper tail below x = a + 1 at shapes 1 to 20 is (0.5 - P) + 0.5, not 1 - P;
  - the density at a shape above 1 and x/scale up to 1 is the same leading factor times e^-y times a / x.
  The cut at x/scale = 1 and the shape bound 20 are existing structural cuts (`quantile_tail`, `DIRECT_SHAPE_MAX`),
  not new fits: no constant is chosen by scoring, and quality/fitted.json does not move.
- **Evidence** (T-3412 Outcome; docs/research/t3412): a 30% hash hold-out on cells with Excel's own arguments, chosen
  before scoring, and three fresh sweeps filled after each freeze (decision 0188 R4.2; commits 5d8b1b79, 56bf7cc5,
  2e1d1dbc): t3412-fresh-sweep (280 cells; GAMMA.DIST cdf 90 of 90 within 4 ulp against 13 for the pair),
  t3412-fresh-sweep-2 (110 cells, shapes 20 to 170) and t3412-density-sweep (120 cells; 77 bit-exact against 33).
- **Eval, per cell against the T-3469 head a9735e10:** gated 34 fixed, 16 closer, 8 visibly fixed, 3 lost; Gate B 465
  to 469 of 506. Ungated 270 fixed, 208 visibly fixed.
- **The regressed gated cells**, each numeric, within 15 digits and 1 ulp from Excel, the form's own residue (it is
  bit-exact at about half its cells and within 1 ulp at 95%):
  - gen-m9/35e8168aae2b00f8 Calc!A119 `CHISQ.DIST(1.5,DURATION(..),TRUE)`: Excel
    0.04050543974481387, ours 0.040505439744813874;
  - gen-m9/a790a4d4a6a82756 Calc!A6 `CHIDIST(MAX(Data!D10:D10,PHI(Data!E11)),3)` = CHIDIST(0.3989…,3): Excel
    0.940460908206416, ours 0.9404609082064159;
  - gen-m9/ce312817dc9c7f0b Calc!A62 `POISSON.DIST(6,2.5,"true")`: Excel 0.9858126880090866, ours 0.9858126880090867.
- **The regressed probes-suite cells** (probes has a baseline; neither cell is visibly off, and 53 other probes cells
  move closer to Excel against these 2 farther):
  - probes/t2753-gamma-density-ratio-sweep S!A18 `GAMMA.DIST(1E-50,2,1,FALSE)`: Excel 9.99999999999989E-51, ours
    9.999999999999891E-51, bit-exact to 1 ulp (a new baseline line);
  - probes/t2753-gamma-density-switch S!A17 `GAMMA.DIST(1E-17,2,1,FALSE)`: 1 to 2 ulp (already a known failure).
- **Ungated cells newly visibly off (20), not covered by the ruling below:** all are probes-numeric/
  t3311-gamma-inv-lower-tail-cdf cells that evaluate GAMMA.DIST or CHISQ.DIST at a column-A quantile (`A15-4*M15`,
  `A63`, `A85+B85`, …) which the engine computes 1 to 4 ulp from Excel's quantile (16 cdf cells, 4 densities). At
  Excel's own quantile the new cdf form is within 1 ulp of Excel at all 16 cdf cells (bit-exact at 5), so Excel
  agrees with the form; the miss is the quantile's, which T-3403 owns. They are ungated, Phase 1b evidence (decision 0188 R1(c)).
- **Owner ruling.** On 2026-11-07, in this session (https://claude.ai/code/session_017265sRpNacu3bZTBp3fnoU), asked
  about the gated cells that leave bit-exactness (the three gen-m9 cells, and T-3403's), the owner chose "Allow, with
  records"; asked then about the two probes cells by name, the owner chose to allow them and add them here.
- **Decision:**
  - `rustsheet-eval update-baseline --allow-regressions` runs once in the T-3412 PR, for the three gen-m9 cells and probes S!A18 alone.
  - T-4959 owns the cdf at 1 < x/scale < a + 1 and T-4960 the density's subnormal flush; the cells' 1-ulp
    residue is the form's (T-3242's for shapes below 1).
- **Consequences:** eval/baseline/gen-m9.txt gains exactly the three gen-m9 lines and eval/baseline/probes.txt the S!A18 line, and both
  lose the fixed cells' lines;
  reviewers confirm the baseline diff adds no other line.
