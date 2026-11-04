# T-3754: F tails at a small d1, a huge d2 and x near 1

**Question.** probes-numeric/t3354-f-dist-sweep S!C168 `F.DIST.RT(1.09080885395379,8,5567623)` was 1.4E-10 relative off
Excel, which is correctly rounded there. Which branch was inaccurate, and how wide is the band?

**Branch.** F's incomplete beta is K(4, 2783811.5, u, v) with u about 1.57E-6. bratio swaps it (λ < 0) to a0 = 2783811.5,
b0 = 4, x0 = v, and with b0 < 40 and b0·x0 > 0.7 it takes L140: 3 bup terms, then bgrat from c = 1 (x0^a0). The port has
that branch (`bratio/bup.rs` `l140_bup_bgrat`, T-3568) but refused a0 above `MAX_BUP_A0` = 3000, the range T-3568's
probes covered, so the call fell back to today's Cephes kernel on df1 x/(df1 x + df2), which is the 1E-10 error.

**Method.** [`grid.py`](grid.py) writes a 2,016-cell case: `F.DIST(x,d1,d2,TRUE)` and `F.DIST.RT(x,d1,d2)` for d1 in
{1..8, 10, 13, 16, 20}, d2 in {1E5, 3E5, 1E6, 3E6, 1E7, 1.5E7, 2E7, 2.5E7, 3E7, 1E8, 3E8, 1E9} and x in {0.5, 0.75, 1,
1.09080885395379, 1.25, 1.5, 2}, with 50-digit mpmath `betainc` values as the expected values (no oracle: an accuracy
measurement). `rustsheet-eval show --all --root ROOT` prints the engine's values and `grid.py score` the largest relative
error per stratum. "Before" is main b71347b16 (cap 3000), "after" the cap at 1E7.

**Result.** The inaccurate band is d1 >= 4 (b0 >= 2, where bup takes terms and the cap applies) with d2 from 1E5 to 1.5E7
(d2 of 2E7 and up was already accurate for d1 up to 20; d1 of 1 to 3 never took the capped composition). In it the
fallback is 1.6E-12 to 3.7E-10 off; with the cap at 1E7 every stratum of the grid is under 1E-14 (largest 8.0E-15).

| tail | d1 | d2 | max rel. error, before | max rel. error, after |
|---|---|---|---|---|
| cdf | 1-2 | 100000 | 2.1e-16 | 2.1e-16 |
| cdf | 1-2 | 300000 | 6.4e-16 | 6.4e-16 |
| cdf | 1-2 | 1e+06 | 1.3e-15 | 1.3e-15 |
| cdf | 1-2 | 3e+06 | 2.1e-16 | 2.1e-16 |
| cdf | 1-2 | 1e+07 | 1.1e-15 | 1.1e-15 |
| cdf | 1-2 | 1.5e+07 | 1.5e-15 | 1.5e-15 |
| cdf | 1-2 | 2e+07 | 1.1e-15 | 1.1e-15 |
| cdf | 1-2 | 2.5e+07 | 1.9e-15 | 1.9e-15 |
| cdf | 1-2 | 3e+07 | 1.7e-15 | 1.7e-15 |
| cdf | 1-2 | 1e+08 | 6.4e-16 | 6.4e-16 |
| cdf | 1-2 | 3e+08 | 4.3e-16 | 4.3e-16 |
| cdf | 1-2 | 1e+09 | 1.5e-15 | 1.5e-15 |
| cdf | 10-20 | 100000 | 1.6e-12 | 3.1e-15 |
| cdf | 10-20 | 300000 | 4.4e-12 | 1.4e-15 |
| cdf | 10-20 | 1e+06 | 2.9e-11 | 2.2e-15 |
| cdf | 10-20 | 3e+06 | 1.3e-10 | 2.6e-15 |
| cdf | 10-20 | 1e+07 | 1.2e-10 | 2.4e-15 |
| cdf | 10-20 | 1.5e+07 | 1.4e-10 | 2.6e-15 |
| cdf | 10-20 | 2e+07 | 1.7e-15 | 2.2e-15 |
| cdf | 10-20 | 2.5e+07 | 2.2e-15 | 2.2e-15 |
| cdf | 10-20 | 3e+07 | 2.7e-15 | 2.7e-15 |
| cdf | 10-20 | 1e+08 | 2.0e-15 | 2.0e-15 |
| cdf | 10-20 | 3e+08 | 1.7e-15 | 1.7e-15 |
| cdf | 10-20 | 1e+09 | 2.0e-15 | 2.0e-15 |
| cdf | 3-9 | 100000 | 3.7e-12 | 5.8e-16 |
| cdf | 3-9 | 300000 | 1.1e-11 | 1.2e-15 |
| cdf | 3-9 | 1e+06 | 4.7e-11 | 2.0e-15 |
| cdf | 3-9 | 3e+06 | 1.5e-10 | 1.3e-15 |
| cdf | 3-9 | 1e+07 | 1.3e-15 | 1.3e-15 |
| cdf | 3-9 | 1.5e+07 | 1.2e-15 | 1.2e-15 |
| cdf | 3-9 | 2e+07 | 2.3e-15 | 2.3e-15 |
| cdf | 3-9 | 2.5e+07 | 1.3e-15 | 1.3e-15 |
| cdf | 3-9 | 3e+07 | 6.0e-15 | 6.0e-15 |
| cdf | 3-9 | 1e+08 | 2.5e-15 | 2.5e-15 |
| cdf | 3-9 | 3e+08 | 2.4e-15 | 2.4e-15 |
| cdf | 3-9 | 1e+09 | 2.0e-15 | 2.0e-15 |
| rt | 1-2 | 100000 | 4.3e-16 | 4.3e-16 |
| rt | 1-2 | 300000 | 5.8e-16 | 5.8e-16 |
| rt | 1-2 | 1e+06 | 1.4e-15 | 1.4e-15 |
| rt | 1-2 | 3e+06 | 6.3e-16 | 6.3e-16 |
| rt | 1-2 | 1e+07 | 1.9e-15 | 1.9e-15 |
| rt | 1-2 | 1.5e+07 | 2.0e-15 | 2.0e-15 |
| rt | 1-2 | 2e+07 | 1.6e-15 | 1.6e-15 |
| rt | 1-2 | 2.5e+07 | 2.1e-15 | 2.1e-15 |
| rt | 1-2 | 3e+07 | 2.0e-15 | 2.0e-15 |
| rt | 1-2 | 1e+08 | 7.2e-16 | 7.2e-16 |
| rt | 1-2 | 3e+08 | 5.8e-16 | 5.8e-16 |
| rt | 1-2 | 1e+09 | 1.5e-15 | 1.5e-15 |
| rt | 10-20 | 100000 | 3.6e-12 | 5.6e-15 |
| rt | 10-20 | 300000 | 8.7e-12 | 4.4e-15 |
| rt | 10-20 | 1e+06 | 8.6e-11 | 4.0e-15 |
| rt | 10-20 | 3e+06 | 2.3e-10 | 5.1e-15 |
| rt | 10-20 | 1e+07 | 2.8e-10 | 8.0e-15 |
| rt | 10-20 | 1.5e+07 | 3.7e-10 | 4.6e-15 |
| rt | 10-20 | 2e+07 | 2.0e-15 | 5.0e-15 |
| rt | 10-20 | 2.5e+07 | 1.5e-15 | 1.5e-15 |
| rt | 10-20 | 3e+07 | 3.1e-15 | 3.1e-15 |
| rt | 10-20 | 1e+08 | 2.5e-15 | 2.5e-15 |
| rt | 10-20 | 3e+08 | 1.5e-15 | 1.5e-15 |
| rt | 10-20 | 1e+09 | 2.5e-15 | 2.5e-15 |
| rt | 3-9 | 100000 | 6.7e-12 | 1.2e-15 |
| rt | 3-9 | 300000 | 2.8e-11 | 1.6e-15 |
| rt | 3-9 | 1e+06 | 8.2e-11 | 2.6e-15 |
| rt | 3-9 | 3e+06 | 2.6e-10 | 2.4e-15 |
| rt | 3-9 | 1e+07 | 1.7e-15 | 1.7e-15 |
| rt | 3-9 | 1.5e+07 | 1.6e-15 | 1.6e-15 |
| rt | 3-9 | 2e+07 | 3.1e-15 | 3.1e-15 |
| rt | 3-9 | 2.5e+07 | 1.9e-15 | 1.9e-15 |
| rt | 3-9 | 3e+07 | 8.0e-15 | 8.0e-15 |
| rt | 3-9 | 1e+08 | 3.3e-15 | 3.3e-15 |
| rt | 3-9 | 3e+08 | 3.1e-15 | 3.1e-15 |
| rt | 3-9 | 1e+09 | 2.3e-15 | 2.3e-15 |

**Eval (ungated as well as gated).** `rustsheet-eval run` over probes-numeric, probes, gen-m9 and ironcalc-calc on both
binaries: 35 cells become bit-exact (34 in probes-numeric t4017-f-huge-df2-above-mean-1e7, 1 in t1538-f-quantile-switchover)
and 3 stop being bit-exact, all 1 ulp and within 15 digits (t4017 ...-1e7 A37 `F.DIST.RT(3,4,1E7)`, A97
`F.DIST.RT(1.5,10,1E7)`, A98 `F.DIST(1.5,10,1E7,TRUE)`). Cells outside 15 significant digits of Excel: 93,568 to 93,516
(52 fewer, none new): t3354-f-dist-sweep S!A168 and S!C168, 46 cells of t4017 ...-1e7 and 4 of t1538 (F.INV at a df of
1E5 and 1E6, whose search crosses this tail). `rustsheet-eval check` (gated suites): no regression, nothing fixed.
