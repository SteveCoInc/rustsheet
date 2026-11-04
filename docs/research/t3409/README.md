# T-3409: Excel's gamma inverse search at a large shape

The model (implemented in `crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/huge_shape.rs`):

1. **Start**: Excel's Wilson-Hilferty start x0 (`wilson_hilferty_quantile`, T-2982/T-3070 constants).
2. **Give-up value**: where the gamma density at x0 is below the smallest normal double, Excel returns x0.
3. **Step rule**: Newton's method on the odds P/Q = r (r = p/(1-p) for the lower tail, (1-p)/p for the
   upper), step (PQ - r Q^2)/f. From Excel's start (above the quantile) it never overshoots; each step moves
   about 1/|z| standard deviations, so the count is about the fall in ln density from x0 to the quantile
   (through the mode when x0 lies past it).
4. **Step limit**: Excel is the true quantile where a relative step below 1E-8 comes within 207 steps
   (`SEARCH_STEP_LIMIT`, FITTED), else `#NUM!`.

## Replay (the fit)

Scripts: `build.py` (the cells), `feat.py` (start and density features; imports `feat_helpers.py`), `feat_helpers.py`
(Temme's P and Q, the density, `odds_newton`), `run2.py` (the replay), `bound.py` (step counts near the edge) and
`sweep.py` (the fresh sweep's generator).

`build.py` collects every GAMMA.INV/GAMMAINV/CHISQ.INV/CHISQ.INV.RT/CHIINV cell of probes/t2982-*, t3070-*,
t3350-*, t3396-* and probes-numeric/fit-audit-t3070-*, t3350-r4-* with its oracle value; `feat.py` adds the start and
its density (40-digit mpmath, Temme's leading term for P and Q); `run2.py SHAPE_LO SHAPE_HI` replays the search
(`feat_helpers.odds_newton`) and `bound.py` gives the step at which the relative step first falls below 1E-6 ... 3E-16
for the cells near the edge. Run them from a scratch directory with the tools venv (`mpmath`).

- Rule 2 (start where its density underflows) picks exactly the 197 cells where Excel is its start, among 1413
  cells from shape 1E5 (the other 20 cells below the cut are `CHISQ.INV` past df 1E10, a domain error).
- Rule 4, by the count at 1E-8 and limit N, train = probes/ (869 rows, a formula in two cases counted twice),
  held-out = probes-numeric/ (314 rows):

  | N | train | held-out |
  |---|---|---|
  | 205 | 857/869 | 310/314 |
  | 206 | 861/869 | 310/314 |
  | **207** | **869/869** | **313/314** |
  | 208 | 865/869 | 314/314 |
  | 209 | 860/869 | 313/314 |

  The split was named after the replay had been read, so the held-out column is not a clean hold-out: the fresh
  sweep below is the out-of-sample test. The one held-out miss at 207 is fit-audit-t3070-band-upper A39
  `GAMMA.INV(1.11E-10,9000000000,1)` (a value at 208; T-4206).
- From shape 1E5 to 1E8 all 282 cells converge in far fewer steps (all values, as Excel).
- The tolerance is not a fit: the counts move together by at most a step between 1E-8 and 3E-16.

## Hash split (review round 2, decision 0188 R4.1)

`split.py`, stated before it was scored: a cell (distinct formula) is held out when the first hex digit of
sha1(formula) is 0 to 4 (5/16). The limit is chosen on the train part only, at the 1E-8 tolerance:

| | train (703) | held-out (336) |
|---|---|---|
| model, N = 207 (the unique train-best N; 205: 694, 206: 696, 208: 700, 209: 696) | 702 | **336** |
| the correctly rounded candidate (the true quantile at every cell) | 465 | 220 |

Margin: N = 207 is the only train optimum, and one step either way costs 2 to 6 train cells. The earlier
split by suite (probes/ against probes-numeric/: 869/869 and 313/314) was named after the replay was read.

Alternatives ranked out: plain Newton on P from Excel's start overshoots past 0 on its first step wherever the start
lies past the mode, yet Excel is a value at 195 such cells (from shape 1E8). The fixed cuts it replaces (T-3070's upper
edge, T-3350's lower edge) are main's model, scored below.

## Fresh sweep (decision 0188 R4)

`sweep.py generate` drew probes-numeric/t3409-r4-gamma-inv-search-sweep (280 cells, seed 3409) after the model was
written. The limit was 208 at the draw (4a760bf6a) and was fixed at 207 (ab098d7c0) before the fill was read, because
the replay's step count had been read at the wrong tolerance column (the draw does not use the model). `sweep.py
generate-wide` drew the second stratum probes-numeric/t3409-r4-gamma-inv-search-sweep-wide (120 cells, shapes 1E5 to
3E8 and 1.3E10 to 1E12) after review round 1, with the model as merged. Scored with the engine (doubles, Temme's
`temme_pair`); "envelope" is Gate A's (within 15 digits of Excel, or 1E-9 of the math sidecar):

| | types right | within 15 digits | envelope |
|---|---|---|---|
| sweep, main (T-3070/T-3350 band cuts) | 254/280 | 243/280 | 247/280 |
| sweep, T-3409 | **278/280** | 267/280 | **271/280** |
| wide, main | 120/120 | 119/120 | 119/120 |
| wide, T-3409 | 120/120 | 119/120 | 119/120 |

Outside the envelope with T-3409:
- sweep A177 and A191: the type, a step from the limit (T-4206).
- sweep A226, A231, A262, A268, A276, A278, A279, and wide A63: Excel's start, where ours (like main) is the true
  quantile. Below shape 1E9 the start's fitted higher terms are 1E-9 to 3.4E-9 off, so the start rule stays at 1E9 and
  up (`START_SHAPE_MIN`); the math sidecar has no value for these cells ("no single value"), so neither form is inside
  (T-4205; A226, at shape 4.8E9, is T-3272's closed form).
