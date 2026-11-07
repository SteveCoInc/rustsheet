# T-3740: BINOM.DIST.RANGE past the mean, fresh sweep predictions

The form under test is frozen before the fill. Past the walk's budget, when the lower end is past the mean
(`s1 - 1 > n·p`), the range is `P(X > s1 - 1) - P(X > s2)`, where each tail is the `w` of the bratio call whose `w1`
is BINOM.DIST's cdf.

Probe: probes/t3740-binom-range-past-mean-sweep, built by `gen.py` here with seed 374020261107 and drawn after the
freeze.
- Rows 1-40 start at mean + 0.1..3 sd, the band that t3444 never probed.
- Rows 41-60 start at mean + 3..8 sd.
- Rows 61-70 start just below the mean. They are controls, still on the cdf difference.
- Column A is BINOM.DIST.RANGE. Column B is Excel's own `BINOM.DIST(s2)-BINOM.DIST(s1-1)`, which is the engine's old
  form.

`predictions.tsv` holds the engine's A (new form) and B (old form) for every row. It was committed before the
excel-graph fill.

Predictions:
1. Rows 1-60: Excel's A is within 1E-12 relative of the engine's new A.
2. Rows 1-60: the engine's new A is at least as close to Excel's A as the engine's cdf difference B is, on every row.
3. Rows 61-70: the engine's A equals its B (the old path), and Excel's A equals Excel's B to the bit.
4. Some rows in 1-60 have Excel's A differing from Excel's B, as on t3444's 21 rows. This would confirm that Excel does
   not use its cdf difference past the mean.

Nothing here is tuned. If a prediction fails, the form is not changed to fit the fill. The result is recorded and the
task follows decision 0188 R4.
