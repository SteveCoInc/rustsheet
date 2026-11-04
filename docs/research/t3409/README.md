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

## Fresh sweep (decision 0188 R4)

`sweep.py` drew probes-numeric/t3409-r4-gamma-inv-search-sweep (280 cells, seed 3409) after the model was written,
and the limit was fixed at 207 (commit ab098d7c0) before its oracle fill was read. Scored with the engine
(doubles, Temme's `temme_pair`):

| | type right | within 15 digits |
|---|---|---|
| main (T-3070/T-3350 band cuts) | 254/280 | 243/280 |
| T-3409 (search model) | **278/280** | 267/280 |

The two type misses (A177, A191) are at the edge (T-4206). The 11 digit misses are Excel's start, with the right type,
where the start's fitted higher terms are 1E-9 to 3E-9 off below shape 1E9 (T-4205; A226 at 4.8E9 is T-3272's range).
