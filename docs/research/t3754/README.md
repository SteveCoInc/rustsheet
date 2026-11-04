# T-3754: the F tails at a small d1 and a huge d2

Question: why `F.DIST.RT(1.09080885395379,8,5567623)` was 1.4E-10 off while Excel is correctly rounded, and how
far the band reaches.

Branch: bratio's dispatch sends (a, b) = (4, 2783811.5) to L140 bup+bgrat (a0 = 2783811.5, b0 = 4), which the port
leaves above a0 = 3000 (`MAX_BUP_A0`), so `incomplete_beta::beta_tails` answers. There `large_shape_lower` (the
DiDonato-Morris expansion) was skipped because b/a = 1.4E-6 was above its 1E-6 cut, and Cephes' continued fraction
times `large_prefactor` answered, right to about 10 digits. The same happened for a below 1E6 (d2 below 2E6).

Fix (crates/rustsheet-engine/src/functions/statistical/large_shape_beta.rs):
- the expansion applies from a = 1E4 (was 1E6) with b/a up to 1E-3 (was 1E-6);
- Q(b, z)/r is Legendre's fraction from z = 0.9 b (was b + 1). `incomplete_gamma`'s Q over r is up to 2.4E-15 off
  between b and b + 1; at the task's point (b = 4, z = 4.36) it left the tail 5 ulp off Excel's correctly rounded
  value. Below 0.9 b the fraction cancels (1E-11 off at b = 4, z = 0.1), so `gamma_ur` stays there. The cut was
  set from a 40-digit check of both at b from 0.1 to 39.5 and z from 0.85 b to b + 1 (fraction within 7E-16 from
  0.9 b up).

Grid (`grid.py`, `grid_dump.rs`; d1 2 to 20, d2 2E4 to 1E9, x 0.5 to 2, against mpmath `betainc` at 50 digits at the
exact u of the double inputs): `before.out` (main) and `after.out`, raw engine values in `engine-before.txt` and
`engine-after.txt`. The d1 <= 10 band went from up to 4.5E-10 to under 1E-14 (9.8E-15 at most). The only points
above 1E-14 after are at x = 1 exactly with d1 = 12, where bratio's ported bfrac answers (unchanged by this task;
T-3479's residue).

Fresh check on cells not used to set the cuts: probes-numeric t4017-f-huge-df2-above-mean-* against their committed
120-digit references (docs/research/t4017/references.json). Every F.DIST.RT cell whose value moved is closer to the
math than on main, except `F.DIST.RT(1.5,4,3E7)` (0.4 to 0.6 ulp); at d1 of 12 to 79 and d2 = 1E7 or 3E7 main was
1E3 to 2.7E6 ulp off and the branch is 0.1 to 424 ulp off (Excel: 2.4 to 465). Eval: 16 cells newly pass, 1 newly
off by ulps: `F.DIST.RT(1.2128860145816052,5,1E10)` (probes-numeric t1538-beta-dist-at-excel-inv A23; x is cut to
1.2128860145816 on entry), where Excel and main are 4.0 ulp off the math and the branch 1.0.

Reproduce: see the docstrings of `grid.py` and `grid_dump.rs`.
