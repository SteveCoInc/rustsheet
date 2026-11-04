# T-4119: F tails at df2 2E6 to 5E7, the engine against 60-digit math

`grid.py` writes `grid-references.json`. Each entry is `[upper, x, df1, df2, value]`, computed with mpmath 1.3 at 60
digits:
- upper: `true` for F.DIST.RT, `false` for F.DIST(.., TRUE);
- the axes: df1 in 3, 5, 8, 12, 20, 40, 60, 79; df2 in 2.1E6, 5E6, 1E7, 2E7, 5E7; x in 0.5, 0.9, 1, 1.1, 1.5, 2, 3, 5;
- a tail is kept only where it is above 1E-120. That leaves 640 points.

**How the reference is computed.** Each value is the tail at the *exact* ratio d2/(d2 + d1·x), not at the rounded
double Excel and the engine form. At df2 1E7 the two differ by about 2E-15 relative. So errors near 1E-14 below are
partly that rounding, not the kernel. docs/research/t4017/references.json is computed at the double arguments, and the
unit test uses it.

The engine (`fisher::f_tail`) was measured on main after T-3754, which raised bup+bgrat's cap to a0 <= 1E7. "Before" is
with large_shape_beta's b/a <= 1E-6 cut; "after" is with the cut removed (T-4119). Worst relative error per df2:

| df2 | before | after |
|---:|---:|---:|
| 2.1e+06 | 2.8e-11 | 2.1e-14 |
| 5e+06 | 1.2e-10 | 2.0e-14 |
| 1e+07 | 1.6e-10 | 1.5e-14 |
| 2e+07 | 5.1e-10 | 6.8e-15 |
| 5e+07 | 7.0e-10 | 1.8e-14 |

Per df1:

| df1 | before | after |
|---:|---:|---:|
| 3 | 2.0e-15 | 2.0e-15 |
| 5 | 2.4e-15 | 2.4e-15 |
| 8 | 4.5e-15 | 4.5e-15 |
| 12 | 5.0e-15 | 5.0e-15 |
| 20 | 8.0e-15 | 8.0e-15 |
| 40 | 5.1e-10 | 7.8e-15 |
| 60 | 3.0e-10 | 1.8e-14 |
| 79 | 7.0e-10 | 2.1e-14 |

Over all 640 points the worst error falls from 7.0e-10 to 2.1e-14, and the median from 5.0e-16 to
3.4e-16. Points past 5E-14 fall from 94 to 0.

Before, every point past 5E-14 was at df1 40, 60 or 79, at every df2 of the grid. Those calls skip T-3754's L140
bup+bgrat port, which takes at most `MAX_BUP_TERMS` = 9 bup terms (b0 = df1/2 below 10) and a0 <= 1E7. The large-shape
expansion's b/a cut (b/a = df1/df2 above 1E-6) then sent them to Cephes' continued fraction, 1E-11 to 7E-10 off. At
df1 3 to 20 the worst point was already 8E-15 before.

## BETA.DIST at shapes the F grid does not reach

The same function also serves BETA.DIST and the quantile searches, so `beta-dist-grid-references.json` checks the rest
of the widened region. Each entry is `[x, a, b, I_x(a, b)]`:
- a in 1.2E6, 3E6, 1E7, 2.5E7, 3.9E7;
- non-half-integer b in 0.7, 2.3, 7.9, 13.37, 25.6, 39.4, with b/a at most 4E-5;
- x at the mean and 1 and 3 standard deviations either side;
- 60-digit mpmath, kept above 1E-120.

That leaves 145 points, measured through `bratio::beta_dist_lower_tail`, BETA.DIST's kernel. Before: median 4.0E-16,
worst 4.9E-10, 30 points past 5E-14. After: median 3.9E-16, worst 4.9E-15, none past 5E-14. No point gets worse by more
than a factor of 1.5.
