# T-3886: bpser's remaining items: a = b = 1/2 and the 6 shape-of-1 cells (negative results)

T-3886 took four items over from T-3192. Two are recorded here. Neither is ported. The other two (the b0 >= 8 L diff
and T-2505's 987 moved cells) need the deleted T-2464 harness and a rebuilt bindings venv before anything can be
scored, so they moved to T-4070.

## 1. BETA.DIST at a = b = 1/2: no closed form beats the ported direct form

`score_half_x87.py` (output in `score_half_x87.out`) scores Excel's B column on the three sweeps
(probes-numeric/t3192-beta-half-half, -2 and -3: 321, 341 and 300 cells; excel-graph) against:
- 10 double-only forms. These are atan, atan2, asin and acos, with sqrt(x)/sqrt(y), sqrt(x/y) and the
  reassociations of 2/pi.
- 192 precision variants. Each of t, the angle, the result arithmetic and pi is held to 53 or 64 bits, under three
  angle forms and four result forms. The 64-bit cases model an x87 register that is never stored.

| form | sweep 1 | sweep 2 | sweep 3 | total |
|---|---|---|---|---|
| direct `2*atan(sqrt x/sqrt y)/pi` (ported), and every precision variant equal to it | 270 | 228 | 239 | 737 |
| `atan(..)/(pi/2)` | 270 | 228 | 239 | 737 |
| `2*atan2(sqrt x, sqrt y)/pi` | 252 | 221 | 236 | 709 |
| `2*atan(sqrt(x/y))/pi` | 236 | 215 | 228 | 679 |
| `2*asin(sqrt x)/pi` | 247 | 215 | 204 | 666 |
| `atan(..)*(2/pi)` | 231 | 193 | 189 | 613 |
| `0.5+asin(2x-1)/pi`, `acos(1-2x)/pi`, `1-2*acos(sqrt x)/pi` | 92-157 | 134-213 | 78-143 | ≤ 491 |

- No variant beats the direct form's 737. The extended-precision variants either equal it or score lower.
- At x = 0.5, Excel gives 0.5000000000000001. Of the 192 variants, 22 give that value, and the best of them is the
  asin form at 666. Nothing that reproduces x = 0.5 also fits the other cells.
- Excel's worksheet `2*ATAN(SQRT(x)/SQRT(1-x))/PI()` (sweep 1, column D) equals the double direct form on all 321
  rows. Excel's ATAN, SQRT and PI therefore agree with libm's on these points.
- So the 40 to 54 misses per sweep are not a libm difference in the worksheet functions. BETA.DIST takes some other
  route that has no closed form among the ones tried.
- Excel's off cells sit a scattered 1 to 2 ulp from the correctly rounded value in the middle of the range. Above
  about 0.9 they sit on it, which is why T-3192's 0.9 switch scores 855 of 962.

## 2. The 6 cells T-3170's shape-of-1 w moved off: no single variant of w fits them

Branch: `incomplete_beta::shape_one_tails`, w = -excel_expm1(b0 * excel_ln_one_minus(x0)), w1 = (0.5 - w) + 0.5.
The intermediates were dumped with a test-only instrument that was not committed. The cell inputs after the swap are
in `ln1m_tie.py`.

Per cell, the variant that gives Excel's bits. A blank means none of excel_expm1, libm expm1, 1 - Excel's EXP, or the
Kahan reassociations ((u-1)*(p/ln u), (u-1)/(ln u/p) and the extended-quotient forms) does at our l and p:

| cell | Excel | ours | matched by, at our l and p | at l = correctly rounded |
|---|---|---|---|---|
| t2220-f-dist-sweep-cdf A191 | 0.0950722141742486 | ...862 | 1 - EXP only | excel_expm1 (ours) |
| t2220-f-dist-sweep-cdf A195 | 0.7718929202102462 | ...464 | libm expm1, 1 - EXP, Kahan reassociated | (l is CR) |
| t2220-f-dist-sweep-rt A195 | 0.22810707978975375 | ...364 | libm expm1, 1 - EXP, Kahan reassociated | (l is CR) |
| t2220-f-dist-sweep-d2-1 A16 | 0.00011697947050321284 | ...283 | libm expm1 only | (l is CR) |
| t1745-beta-inv-roundtrip G145 | 0.04999999999999938 | ...949 | none | every variant |

- `ln1m_tie.py` (output in `ln1m_tie.out`) places each cell's 64-bit r + s inside its 53-bit ulp.
  - A191 sits at 0.553, a near-tie of the kind behind the log1p model's 27 known misses. Excel's l there is the
    correctly rounded value, and with it our expm1 is bit-exact.
  - G145 sits at 0.045, not a tie, yet it also needs the correctly rounded l. The flip there comes earlier in the
    series (r or a term), or from a different ln1p on this path, and is unexplained.
- probes/ftest-decomposition B12 forms x in its caller and was not analysed separately.
- Full-eval A/B of the one variant that fixed the A195 pair, Kahan reassociated (`kahan_reassociated.patch`,
  `docs/research/t3007/cell_diff.py`, both reports at origin/main b71347b1):
  - 134 cells change. 78 become bit-exact and 47 lose bit-exactness.
  - BETA.DIST: 48 better, 12 worse. F.DIST: 18 better, 22 worse. F.DIST.RT: 15 better, 15 worse.
  - The losses include 5 cells of the gated probes suite (probes/t3354-f-dist-tiny-x-huge-d2 A59, A67, A75, A83,
    A91), so it is not portable.
  - The wins are mostly at large b0. They include all 47 t3568-r5-bup column-B cells that T-3764 owns and 24
    t4017-f-huge-df2 cells. The losses span small and large b0 alike (b0 from 1/2 to 1E10), so no shape cut
    separates them. This is recorded for T-3764 as a lead, not a rule.
