# T-4857: the F family's equivalence classes, their coverage, and an equivalence-class probe

Owner request (2026-11-07): "do some thorough analysis of the arguments and their types and their ranges, and
thinking about the general shape of the functions involved here, and then come up with a probe that tries really hard
to do a nice job of getting a full set of equivalence classes across all of the different kind of interesting ranges
of the input arguments, so that we can have more confidence that we're capturing all of the interesting points to spot
areas of discontinuities and nonlinearities that might otherwise easily escape our extrapolation confidence."

Scope: F.DIST (cdf and pdf), F.DIST.RT, FDIST, F.INV, F.INV.RT, FINV; F.TEST and FTEST for argument kinds and shapes
only. Every class below is defined once, in code, in `fclasses.py`; this file says where each edge comes from.

| File | What |
|---|---|
| `fclasses.py` | The classes: df, x magnitude, x position, p, kernel branch, density regime, quantile path; tail helpers |
| `cells.py` | Reads every oracle-backed F-family cell of eval/suites and eval/accuracy/tables, with argument kinds and numbers |
| `audit.py` | The coverage audit: per class and per pair counts (`coverage-before.json`, `coverage-after.json`) |
| `report.py` | `COVERAGE.md` from two audits |
| `gen.py` | The seeded generator (seed 4857) of the two probe cases |
| `score.py` | Excel vs math, engine vs Excel, engine vs math per class and pair; regime changes (`TABLES.md`) |

Reproduce (from `tools/`): `uv run python ../docs/research/t4857/audit.py --exclude t4857 --out
../docs/research/t4857/coverage-before.json`, the same without `--exclude` for `coverage-after.json`;
`python3 ../docs/research/t4857/report.py <before> <after> > ../docs/research/t4857/COVERAGE.md`;
`uv run python ../docs/research/t4857/gen.py --write`; `uv run python ../docs/research/t4857/score.py
../docs/research/t4857/coverage-before.json > ../docs/research/t4857/TABLES.md`.

## 1. Argument kinds

The engine reads each scalar argument through `cast_number_ref` (x, df1, df2, p) and the cumulative flag through
`cast_bool_ref` (`fisher.rs` `f_dist_of`, `f_dist_rt_of`, `f_inv_of`); every argument lifts over an array or range
(`lift_call`). F.TEST reads two samples through `evaluate_sample_arg` (`f_test_sample`).

Prior evidence (Excel cells counted by `audit.py`, `coverage-before.json` `kinds` and `kind_cases`) and what
statistics.md and the coercion docs establish:

| Kind | Prior evidence for the F family | Documented rule |
|---|---|---|
| number literal | every position, about 100k cells | - |
| numeric text `"2.5"` | 1 cell per position (t1049-fdist, -fdistrt, -finv, -finvrt, m9-legacy-*, t3185-m9-statistical-argument-kinds-a/-b); none for F.DIST's df or F.INV's df | statistics.md "Thin distribution functions ... (T-1049)" |
| non-numeric text | x and p: 1 cell per function (t1049); df: only through a reference (gen-m9) | `#VALUE!` |
| empty text `""` | 1 cell per position (t3185) | `#VALUE!` |
| TRUE/FALSE typed | x and p: 1 cell per function (t1049); df: 1 cell each for F.DIST df2, F.INV df1, F.DIST.RT; none elsewhere | reads 1/0 |
| boolean through a reference | 1 to 9 cells per position (gen-m9); none for F.DIST df1 and df2 | statistics.md "Booleans from a reference or a formula are numbers ... (T-2255)" |
| empty cell | 1 to 14 cells per position (gen-m9) | reads 0 |
| error typed or `NA()` | 1 cell per position (t3185, stat-overflow-arguments) | the error |
| error through a reference | none for the scalar functions | **unprobed** |
| one-cell range `A1:A1`, one-element array `{2.5}` | none for the scalar functions | **unprobed** |
| multi-cell range or array (the lift) | 1 to 3 cells per position (stat-argument-modes, stat-later-argument-lift, stat-quantile-lift) | lifts (SPEC-0011) |
| `SEQUENCE` lift, mixed-type array elements | none | **unprobed** |
| omitted argument (`F.DIST(,5,10,TRUE)`) | none | **unprobed** |
| padded text `" 2.5 "`, a `TEXT()` result, a computed boolean `(1=1)` | none for these functions | **unprobed** |
| cumulative: numbers 0, 1, 2, -1, 0.5, 1E-300 | 61 cells (gen-m9) | nonzero is TRUE |
| cumulative: `"TRUE"`/`"FALSE"` against `"1"`/`"0"` text | 37 text cells (gen-m9), none separating the two | **unprobed per value** |
| F.TEST samples: number, text, boolean, blank one-cell, error, error element, 2-D | 1 to 23 cells per kind (t2455, t4223, gen-m9) | statistics.md "F.TEST of a single value (T-2455)" and the next entries |
| F.TEST omitted sample | none | **unprobed** (statistics.md says so) |

The kinds probe (probes/t4857-f-kinds, 666 cells) gives every argument of every function all 22 scalar kinds above,
four lift forms read element by element through `INDEX` (a multi-cell range, an array, a `SEQUENCE`, a mixed array
`{1.5;"2.5";TRUE}`) and a `ROWS` of a lifted range; F.DIST's flag 25 kinds and two lifts; F.TEST and FTEST 27 sample
kinds and shapes in each position. Formulas with too few or too many arguments are not probed: Excel refuses them at
entry, and a workbook holding one does not open in Excel for the web (`FileCorruptTryRepair`).

## 2. Domain and range edges

### x (F.DIST, F.DIST.RT, FDIST)

`fclasses.x_magnitude`, with the code or evidence behind each edge:

| Class | Edge source |
|---|---|
| x < 0 | `#NUM!` (`f_dist_of`'s domain check) |
| -0 | a literal `-0` (is the sign kept?) |
| 0 | cdf 0, tail 1; density `#NUM!` at df1 1, 1 at df1 2, 0 above (t1049-fdist A27, A28) |
| subnormal | number text below DBL_MIN reads as 0 (`FDIST(1E-308,6,1E+8)` = 1, t2435); `EXP(-720)` stays subnormal and the density is `#DIV/0!` (t2586-r5-f-edges-a B931); the tails at a subnormal x were unprobed |
| DBL_MIN .. 1E-307 | the smallest normal x; fl(df2/(df1 x)) overflows here for a large df2 (`f_beta_ratio_overflows`, T-2435) |
| 1E-307 .. 1E-100, 1E-100 .. 1E-16 | the deep lower tail (cdf ~ x^(df1/2)) |
| 1E-16 .. 1E-3, 1E-3 .. 0.1, 0.1 .. 10, 10 .. 1E3 | the body; the mode (df1-2)/df1 · df2/(df2+2) and the mean df2/(df2-2) sit near 1 |
| 1E3 .. 1E16, 1E16 .. 1E100, 1E100 .. 1E300 | the upper tail (sf ~ x^(-df2/2)) |
| 1E300 .. DBL_MAX | fl(df1 x) overflows past DBL_MAX/df1: `#DIV/0!` for the cdf and tails, `#NUM!` for the density (T-1290) |
| DBL_MAX | the edge itself, written as `(2-2^-52)*2^1023` |

`x_position` splits x by its smaller tail: deep (< 1E-300), far (1E-300 .. 1E-16, where 1 - tail rounds to 1), tail
(1E-16 .. 1E-3) and body (to the median), on the lower (`lo`) or upper (`hi`) side. `centre_class` marks x within 1%
of the mode or the mean.

### df1, df2

df is truncated, then must be in [1, 1E10] (`f_df_ok`; statistics.md "F-distribution df limit"). Every other edge is
a shape edge in df units, because the kernel runs on (a, b) = (df1/2, df2/2):

| df class | Shape edge | Source |
|---|---|---|
| < 1 | - | `#NUM!` |
| 1 | a = 1/2: bgrat at b = 1/2 (T-3580) | `bratio/mod.rs` `ported_tails_with` |
| 1 < df < 2 | truncates to 1 | T-1049 |
| 2 | a = 1: Excel's closed forms; a0 = 1 with y0 > 0.9 is the open form | `closed_form_tails`, `shape_one_tails` (T-2502, T-3007) |
| 3-15 | small shapes; the density's small-shape constant | `f_density.rs` |
| 16 | a = 8: the density's `LARGE_DF_MIN`; `STIRLING_MIN` | `f_density.rs`, `incomplete_beta.rs` |
| 17-29, 30, 31 | a0 = 15: L140 bup+bup+bgrat (a0 <= 15, unported) against bup+bgrat | `dispatch` |
| 32-78, 79, 80, 81 | b0 = 40: bratio's L140 family against bfrac/basym; `MAX_SECOND_SHAPE` | `dispatch`, `large_shape_beta.rs` |
| 82-199, 200, 201 | a0 = 100: bfrac against basym (λ > 0.03 a0) | `dispatch` |
| 202 .. 1E5 | the quantile's statrs start ends at df 1E5 | `F_QUANTILE_STATRS_DF_MAX` |
| 1E5 .. 2E6 | below `MIN_SHAPE` (shape 1E6 = df 2E6) | T-4434's region 1 |
| 2E6 .. 2E7 | the large-shape expansion; L140 bup+bgrat up to `MAX_BUP_A0` (shape 1E7 = df 2E7) | `large_shape_beta.rs`, `bup.rs` |
| 2E7 .. 1E9, 1E9 .. 1E10 | past `MAX_BUP_A0`; Excel's `#NUM!` band at df1 79, df2 1E10 (T-4044) | statistics.md "Incomplete beta at a huge first shape" |
| 1E10 | the last accepted df | T-1530 |
| > 1E10 | `#NUM!` | T-1530 |

The task brief names 1E6 as `large_shape_beta`'s MIN_SHAPE; MIN_SHAPE is a shape, so its df edge is 2E6. Both are
probed (a df2 sweep from 1E6 to 4E6, and 1999999/2E6/2000001 triples), as are df 8, 15, 40 and 100 inside their
classes. Edges of the combined shape have no class of their own: a + b = 10 (df1 + df2 = 20, `SERIES_MIN_SHAPE_SUM`)
and a + b = MAXGAM (df1 + df2 = 343.2) choose the engine's fallback kernel, and F.TEST's a + b = 200
(`F_TEST_SERIES_SHAPE_MAX`). The kernel classes carry the first two (`fb:series`, `fb:cephes`).

### p (F.INV, F.INV.RT, FINV)

| Class | Edge source |
|---|---|
| < 0, > 1 | `#NUM!`; F.INV.RT and FINV also reject 0 |
| 0, 1 | F.INV(0) = 0, F.INV(1) `#NUM!`, F.INV.RT(1) = 0 |
| subnormal | reads as 0 (numbers.md "Subnormal results become 0"): no class of its own |
| DBL_MIN, 1E-307 .. 1E-200 | T-4103 (Excel's search far from the math), T-2435's saturation at df2 < 2 |
| 1E-200 .. 1E-20 | below `F_QUANTILE_STATRS_P_MIN`: no statrs start, bisection |
| 1E-20 .. 1E-15 | statrs' estimate kept unrefined (`F_QUANTILE_REFINE_P_MIN`, FITTED T-0974) |
| 1E-15 .. 1E-3, 1E-3 .. 0.5, 0.5, 0.5 .. 1-1E-3 | refined crossing on the smaller tail |
| 1-1E-3 .. 1-1E-15, 1-1E-15 .. 1 | 1 - p loses digits; 1 - 2^-53 and 1 - 1E-16 are probed as exact constants |

`quantile_path` adds the engine's path: refine-lower, refine-upper, without a start (`-nostart`), statrs unrefined,
or Excel's BETA.INV mapped to F (`beta-inv`, `beta-inv-flip` for p > 1/2) once either df reaches 1E5 (T-1538).

## 3. Kernel branch boundaries

F's tails call bratio K(a, b, x, y) with a = df1/2, b = df2/2 and Excel's own arguments (statistics.md "F's
incomplete-beta arguments keep r >= 1"): mx = fl(df1·x); r = fl(df2/mx) when mx <= df2, else fl(mx/df2); the smaller
argument is fl(1/fl(1 + r)), the larger fl(r/fl(1 + r)). So the task's x0 = df2/(df2 + df1·x) is the larger argument
when df1·x < df2 and the smaller otherwise. F.DIST's cdf is w and F.DIST.RT and FDIST are w1.

`fclasses.dispatch` mirrors `bratio/mod.rs` `dispatch` line for line:

- x = 0 or y = 0: Zero or One. Tiny (max(a, b) < 1E-18), Fpser and Apser are unreachable for F (shapes >= 1/2).
- **min(a, b) <= 1** (a df of 1 or 2): swap when x > 0.5; then WBpser when b0 <= 1, W1Bpser when x0 >= 0.3, WBpser
  when x0 < 0.1 and (x0·b0)^a0 <= 0.7, Bgrat when b0 > 15, else BupBgrat (20 bup terms, then bgrat). a0 = 1/2 is
  ported (T-3580). A df of 2 takes Excel's closed forms first (b0 = 1, or a0 = 1 with y0 <= 0.9), and a0 = 1 with
  y0 > 0.9 the open form (`shape1-open`).
- **min(a, b) > 1**: λ = (a + b)·y - b when a > b, else a - (a + b)·x; the tail swap is λ < 0 (x past the mean
  a/(a + b)). Then with b0 < 40 (that shape's df below 80): BpserLarge when b0·x0 <= 0.7; L140 bup+bpser when
  x0 <= 0.7; L140 bup+bup+bgrat when a0 <= 15 (**not ported**: the engine falls back); else L140 bup+bgrat (ported up
  to 9 bup terms and a0 <= 1E7, else the fallback). With b0 >= 40: bfrac when (a0 > b0 and (b0 <= 100 or
  λ > 0.03·b0)) or (a0 <= b0 and (a0 <= 100 or λ > 0.03·a0)), else basym.
- The engine's fallback for the unported branches (`incomplete_beta.rs` `beta_tails`) is an accuracy stand-in, not
  Excel's algorithm: the half-integer series where df1 + df2 is in [20, 343.2), the large-shape expansion where the
  larger shape after the Cephes flip is >= 1E6 and the other < 40, else Cephes' fraction. The kernel classes name it
  (`L140BupBgrat-fb:cephes` and so on).
- Zero and error cuts: L140 bup+bgrat's prefactor underflow is Excel's exact 0 (T-4045); F.DIST's cdf flushes a
  subnormal result (T-4026); the L140 bup sum overflow is `#NUM!` (T-4044).

The density (`f_density.rs`) has its own regimes, `density_class`: side 1 (fl(m x) <= n) or side 2; r <= 0.375
(Excel's log1p series) or above (LN(1 + r)); the small-df or large-df constant (min(df) < 16); and |P| = |c·ln x|
below 4, 4 to 32, or above (T-2586's parity bands). E < -800 and a result below DBL_MIN are 0; a subnormal x is
`#DIV/0!`.

Every one of these conditions is an equivalence-class edge. `gen.py` `find_edges` walks x over 1,500 log-spaced points
from e^-705 to e^709 at 22 df pairs (`EDGE_DF`), and wherever the label (`kernel_class` with the swap flag, or the
density's side, form and constant) changes it bisects on the doubles' bit patterns to the two adjacent doubles where it
changes. Each edge (at most two per transition type and group) gets 8 points: one ulp below the last double of the old
branch, that double, the first double of the new branch, one ulp above it, and 1E-9 and 1E-3 relative either side (all
exact constant cells). The df and p edges get the values that straddle them (section 2).

## 4. Shape of the functions, and where Excel could switch unseen

- **Smoothness.** The F cdf is analytic in x > 0 and in the df. Any jump, kink or sudden loss of digits along a 1-D
  line is an algorithm switch (Excel's or the engine's): the dispatch edges of section 3, the zero and overflow cuts,
  df truncation, and Excel's own constants we have not met.
- **Tails.** The lower tail behaves like x^(df1/2) near 0 and the upper like x^(-df2/2) at infinity. At df1 1 the lower
  tail cannot go below about 1E-154 for a normal x, nor the upper below that at df2 1, so the deep bands are
  unreachable there (`COVERAGE.md`'s "reached" counts).
- **Cancellation.** Excel forms the larger beta argument as r/(1 + r), not as 1 minus the smaller, so the complement
  keeps its digits until r reaches 2^53, from where the larger argument is 1 and only the smaller carries information.
  The quantiles at p near 1 form 1 - p; BETA.INV at a huge df keeps only y's distance from 1 on the 2^-53 grid
  (T-1538).
- **Underflow and overflow.** Five cuts in x per df pair: fl(df1·x) overflow (`#DIV/0!`), fl(df2/(df1·x)) overflow
  (`#NUM!`), the smaller beta argument 1/(1 + r) leaving the normal range (section 6: Excel's tail is then 0), a
  subnormal result (flushed), and the density's E < -800.
- **Symmetry.** F(x; d1, d2) = 1 - F(1/x; d2, d1): F.DIST(x, d1, d2, TRUE) and F.DIST.RT(1/x, d2, d1) are the same
  incomplete beta with the shapes swapped, and in Excel's r >= 1 form the same r up to the rounding of 1/x. The probe
  pairs 70 of them, and 30 F.INV(p, d1, d2) with F.INV.RT(p, d2, d1) (whose product is 1).
- **Where Excel could switch without our knowing.** The df between 200 and 2E6 (bfrac and basym at large shapes, where
  the engine has its own kernels), the unported L140 bup+bup+bgrat (a0 <= 15) and L140 bup+bgrat past 9 terms, the
  engine's a + b = 10 and MAXGAM cuts, the quantile's statrs window (1E-20 .. 1E-15) and df 1E5 switch, and the
  density's df 16 constant switch. The probe spends most of its edge and sweep cells there.

## 5. Coverage audit (before)

`audit.py` reads 114,558 oracle-backed Excel cells of the family, 89,908 of them F.DIST densities from the T-2586
research. xlsx suites (ironcalc-calc's `F_DIST.xlsx` and `F_TEST.xlsx`, calipers, corpus-bench, poi) are not read:
tools never parse xlsx values, and the corpora are not checked out here. Full tables: `COVERAGE.md`.

Single classes were mostly covered: every x, df and kernel class of the tails had Excel cells, but not the density at
df1 79 (none) or df2 201 (1), nor the quantiles at df 79, 80, 81, 200 and 201 (none) or p = DBL_MIN (6). The risk was
in the pairs:

| Group | Pair table | Pairs | Empty before | Thin (1-2) before |
|---|---|---|---|---|
| tails | df1 x df2 | 441 | 218 | 51 |
| tails | xmag x df1, xmag x df2 | 231, 231 | 101, 92 | 35, 25 |
| tails | xpos x df1, xpos x df2 | 168, 168 | 42, 32 | 29, 16 |
| tails | kernel x xpos | 160 | 63 | 8 |
| density | df1 x df2 | 441 | 210 | 57 |
| density | xpos x df1, xpos x df2 | 168, 168 | 41, 32 | 21, 13 |
| quantiles | df1 x df2 | 441 | 326 | 52 |

The empty df1 x df2 pairs are the edge values (df 16, 30, 31, 79, 80, 81, 200, 201) against everything, and the large
df classes against each other. The research sweeps fixed one df and swept the other, so the corners (both df large,
both near a shape edge) and the quantiles away from df 1 to 78 had no Excel cell. Kernel classes with few cells: L140
bup+bup+bgrat (37, all on the engine's fallback), L140 bup+bgrat past the port on the series fallback (16), and the
ratio overflow (16).

## 6. The probe and its results

### Design and budget

`gen.py` (seed 4857) wrote probes-numeric/t4857-f-classes (3,272 cells, an exception to decision 0267 item 5's
freeze by decision 0338) and probes/t4857-f-kinds (666 cells), both filled by excel-graph on 2026-11-07, with math
sidecars from `rustsheet-accuracy`'s own code (the F definitions in `tools/src/rustsheet_tools/accuracy/defs/fisher.py`:
mpmath at 50 to 80 digits on the exact ratio d1 x/(d1 x + d2)).

| Section | Cells | Why it gets them |
|---|---|---|
| pair-tail, pair-pdf, pair-inv | 441 each | one row per (df1, df2) class pair, x or p chosen greedily for the uncovered (x, df) and (kernel, position) pairs: closes the 218/210/326 empty df pairs of section 5 |
| domain | 168 | every error or degenerate class of each argument on each function (a few cells per class) |
| edge-x | 384 | 48 dispatch edges along x, 8 exact doubles each: the switches section 3 lists |
| edge-df, edge-p | 318, 120 | the df thresholds 15/16/17, 30/31, 79/80/81, 200/201, 1E5, 2E6, 2E7, 1E10 (below, at, above, and a fraction that truncates), and the quantile's p switches |
| sweeps | 701 | where a hidden switch would hurt most: across b0·x0 = 0.7 and x0 = 0.7, across the mean (bfrac's swap), df1 across 80 and 30, df2 across 2E6 (df1 1, 2 and 1000, outside T-4434), tiny p on both tails at (5, 7) and (30, 3), the density across fl(m x) = n, and the df 2 closed form's y0 = 0.9 cut |
| sym, sym-inv | 140, 60 | 70 F.DIST/F.DIST.RT(1/x) pairs and 30 F.INV/F.INV.RT pairs with the df swapped |
| zero | 80 | results that may be subnormal, each with an `=(..)=0` companion |

The large-df corners took most of the pair rows because they had no evidence at all; T-4434's region (df2 1E4 to 1E10
with df1 3 to 400) got only its pairwise share (one cell per class pair), not a sweep.

### Coverage, before and after (`COVERAGE.md`)

| Group | Pair table | Empty before | Empty after | Thin (1-2) before | Thin after |
|---|---|---|---|---|---|
| tails | df1 x df2 (441) | 218 | 0 | 51 | 207 |
| tails | xmag x df1 / df2 (231 each) | 101 / 92 | 2 / 0 | 35 / 25 | 82 / 80 |
| tails | xpos x df1 / df2 (168 each) | 42 / 32 | 2 / 2 | 29 / 16 | 18 / 10 |
| tails | kernel x xpos (160) | 63 | 48 | 8 | 10 |
| density | df1 x df2 (441) | 210 | 0 | 57 | 232 |
| density | xpos x df1 / df2 (168 each) | 41 / 32 | 2 / 2 | 21 / 13 | 21 / 15 |
| quantiles | df1 x df2 (441) | 326 | 0 | 52 | 347 |
| quantiles | p x df1 / df2 (252 each) | 146 / 135 | 0 / 0 | 39 / 46 | 118 / 118 |

Every single class now has Excel cells. The pairs still empty are unreachable (the deep lower band at df1 1, the
deep upper band at df2 1, x above 1E300 at df1 1E9 where fl(df1 x) overflows) or kernel x position combinations the
dispatch cannot produce. A pair with one or two cells is covered, not dense: that is the point of a covering array.

### Agreement (`TABLES.md`, generated by `score.py`)

Per function, counts within 15 / 13 significant digits:

| Function | Cells | Type agree | Excel ~ math | Engine ~ Excel | Engine ~ math |
|---|---|---|---|---|---|
| F.DIST (cdf) | 858 | 857 | 473 / 667 of 679 | 679 / 755 of 764 | 494 / 671 of 679 |
| F.DIST.RT | 872 | 868 | 536 / 706 of 714 | 678 / 762 of 811 | 557 / 703 of 714 |
| FDIST | 215 | 214 | 83 / 113 of 120 | 158 / 171 of 176 | 88 / 115 of 120 |
| F.DIST (pdf) | 721 | 721 | 465 / 686 of 706 | 551 / 698 of 707 | 484 / 688 of 706 |
| F.INV | 425 | 424 | 136 / 201 of 255 | 176 / 304 of 366 | 149 / 238 of 255 |
| F.INV.RT | 425 | 422 | 146 / 213 of 266 | 207 / 299 of 363 | 161 / 247 of 267 |
| FINV | 289 | 288 | 93 / 114 of 152 | 162 / 198 of 237 | 100 / 129 of 153 |
| F.TEST, FTEST | 108 | 104 | 24 / 24 of 24 | 40 / 40 of 40 | 24 / 24 of 24 |

Excel itself is within 15 digits of the math on only 60 to 70% of the tail and density cells (13 digits: 94 to 98%),
the noise decision 0267 describes; the engine follows Excel more closely than the math does on every tail and
density row. The quantiles are where the two part: Excel is more than 1E-4 off the math on 85 of the 673 quantile cells with a math value.

By kernel branch (tails; engine ~ Excel at 15 / 13 digits): L140 bup+bpser 274 / 274, closed forms 76 / 76,
bgrat at b = 1/2 60 / 60, L140 bup+bgrat (ported) 106 / 107, BpserLarge 182 / 209, WBpser 136 / 139, bfrac
320 / 407 of 457, basym 26 / 39 of 41, and the engine's fallback for L140 bup+bgrat past the port on Cephes 4 / 30 of
33, the weakest branch (Excel is within 15 digits of the math there on only 13 of 32 cells). The unported L140
bup+bup+bgrat matched on 8 of 9 cells at 15 digits.

### Regime changes along the 1-D lines

- **Expected and matched**: df above 1E10 turns every function to `#NUM!` exactly at 10000000001 (10000000000.9
  truncates and is a number); a df below 1 is `#NUM!` and 1.9999999 truncates to 1; the ratio-overflow `#NUM!` ends at
  the exact double the engine predicts (A1481/A1482 at (1, 400), A1529/A1530 at (3, 2000)).
- **New: a zero band after the overflow.** Past the ratio-overflow edge, Excel's cdf is 0 while the smaller beta
  argument is subnormal (A1482:A1485); the engine returns 1.19E-153 (statistics.md "F tails at the edges of the double
  range"; T-4881).
- **New: a jump in the quantile.** `F.INV(1E-k,30,3)` leaves the math by ten orders of magnitude between k = 172 and
  k = 181 (A2777 -> A2778), and `F.INV.RT(1E-k,3,30)` with it; at (5, 7) Excel stays on the math to k = 289
  (statistics.md "F quantiles at a tiny p leave the math abruptly"; T-4884).
- **No jump or kink** along x across b0·x0 = 0.7 (120 cells), x0 = 0.7 (50), the mean on bfrac (80), the density's
  side switch (80) or the df 2 closed form's y0 = 0.9 cut (60), nor in df1 across 80 (63) and 30 (18), nor in df2
  across 2E6 at df1 1 and 2 (60): Excel's error against the math stays at the noise level on both sides of each edge,
  and the engine agrees with Excel to 13 digits across them, so these branch edges are not hiding a discontinuity.
  The one smooth drift is df1 1000 across df2 1E6 to 4E6 (engine 1E-12 to 2E-11 off Excel throughout, T-4886).

### Where the engine drifts from Excel in classes with little prior evidence (the overfitting findings)

1. **The large-shape corners of the tails** (df1 x df2 pairs with 0 prior cells): bfrac and basym with one df at
   1E6 to 1E10 and the other at 200 to 1000 or above: engine 1E-12 to 9E-8 off Excel (A315, A434, A2028:A2031,
   A2052:A2055, A2111:A2123, A2658:A2687); T-4434 covers df1 3 to 400 only (T-4886). At (201, 7.2E8) basym the engine
   is the math and Excel is 1.3E-9 off (A313).
2. **The double range's edges** (kernel x position `ratio-overflow`, `deep-hi` at df 80, 200 and 31, x above 1E300):
   three Excel cut rules the engine lacks (T-4881).
3. **Tiny p and df pairs never probed for the quantiles** (p x df pairs: 146 and 135 empty before): Excel far from
   the math or an error on 91 cells (T-4884), the engine wrong where Excel is on the math on 26 (T-4883), and the BETA.INV mapping
   at a df of 1E5 or more off Excel by 1E-13 to 1E-5 on 71 (T-4885).
4. **The density with both df above 1E8**: 3E-12 to 1.8E-10 off Excel, which is itself up to 1.1E-6 off the math
   (T-4887).
5. **Kinds**: the only kind the engine gets wrong is an omitted F.TEST sample (T-4882).

Everywhere else the new classes agree with Excel as well as the old ones did: the L140 bup+bpser, closed-form, bgrat
and shape-one branches match on every new cell, so the evidence that tuned them was not overfitted to its own points.
