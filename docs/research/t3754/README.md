# T-3754: F tails at a huge df2, the engine against 60-digit math

The grid is `grid-references.json`. Each entry is `[upper, x, df1, df2, value]`, written by `grid.py` with mpmath 1.3 at
60 digits:
- upper: `true` for F.DIST.RT, `false` for F.DIST(.., TRUE);
- the axes: df1 in 3, 5, 8, 12, 20, 40, 60, 79; df2 in 2.1E6, 5E6, 1E7, 2E7, 5E7; x in 0.5, 0.9, 1, 1.1, 1.5, 2, 3, 5;
- the tail is kept only where it is above 1E-120. Deep tails need the higher-precision references of
  docs/research/t4017.

That leaves 640 points. On each, the engine's `fisher::f_tail` was measured at the large-shape expansion's old and new
shape-ratio cut (`large_shape_beta::MAX_SHAPE_RATIO`). Worst relative error per df2:

| df2 | cut 1E-6 (before) | cut 4E-5 (after) |
|---:|---:|---:|
| 2.1e+06 | 1.4e-10 | 2.1e-14 |
| 5e+06 | 1.5e-10 | 2.0e-14 |
| 1e+07 | 4.3e-10 | 1.5e-14 |
| 2e+07 | 5.1e-10 | 6.8e-15 |
| 5e+07 | 7.0e-10 | 1.8e-14 |

Over all 640 points the worst error falls from 7.0E-10 to 2.1E-14, and the median from 6.4E-16 to 2.5E-16. Points
past 5E-14 fall from 169 to 0.

Before, b/a above 1E-6 put these inputs on Cephes' continued fraction, which is 1E-11 to 1E-9 off at b of 1E6 and more.
4E-5 is `MAX_SECOND_SHAPE / MIN_SHAPE`, so the expansion now covers every a >= 1E6 and b < 40.
