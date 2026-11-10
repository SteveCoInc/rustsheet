# PROBE-REQUEST val4 — HYPGEOMDIST / BINOM.DIST kernel (r8hyp, 2026-11-10)

Two workbooks, 387 checked formula cells, same `rustsheet-case/v1` layout and the same precision canary sheet as
val3. No expected values and no oracle sidecars are included. Frozen model values for every cell are in
[val4-hyp-predictions.json](val4-hyp-predictions.json): research metadata, **not** oracle values.

| Case | Checked formulas | What it pins |
|---|---:|---|
| `val4-hyp-01` | 165 | The direction of one rounding inside bd0's near-mean logarithm, at exact ties |
| `val4-hyp-02` | 222 | Whether a certain hypergeometric outcome is a shortcut (exactly 1) or the factor product |

## Why

Scoring val3 identified Excel's interior `BINOM.DIST(k,n,p,FALSE)` to the bit on every resolvable cached cell (2,953
cells), and with it `HYPGEOMDIST` as the product of three worksheet binomial factors (2,170 cells). Two things are left
that the cached cells cannot decide.

## val4-hyp-01: exact ties of the log-ratio series

Near the mean, bd0(x, M) takes ln(x/M) from the stored series `2 (v + sum_k v^(2k+1)/(2k+1))`, v = (x-M)/(x+M). In
every row of sheets `P` (36 BINOM.DIST cells, 14 with a dyadic p and 22 with a two-decimal p, n 12 to 89) and `Q` (32 POISSON.DIST cells, k 11 to 385) the
stored sum `v + s` is an **exact tie between two doubles**, and the pmf changes by at least one ulp with the direction
of that single rounding.

- Cached evidence: 5 cached BINOM.DIST cells hold such a tie and all 5 follow "ties to even" (for example
  `BINOM.DIST(3,11,0.5,FALSE)`); one cached cell needs the other neighbour: `HYPGEOM.DIST(12,17,15,21,FALSE)` is
  0.4561403508771925 where the product of the model's factors is ...926, and the product is Excel's value if the factor
  `BINOM.DIST(5,6,12/21,FALSE)` is one ulp lower, which is what the other tie direction gives.
- `P!D38:D41` read that factor directly (p computed in the sheet as `=12/21`, and as the 15-digit literal), and the two
  other factors of the same tuple.
- Sheet `H` is the val2 factor bridge (E `HYPGEOMDIST`, F `HYPGEOM.DIST(..,FALSE)`, G p = MAX(x, n-x)/N, H, I, J the
  three `BINOM.DIST` factors, K = (H/J)*I) on 12 tuples one of whose factors holds a tie. Row 2 is (12,17,15,21).

Outcomes:
- every tie cell equals `ties_to_even`: the kernel is as modelled, and the (12,17,15,21) miss is in the bridge, not in
  the factor (then `H!E2 <> H!K2` shows it directly);
- the tie cells split by some property (p dyadic or not, the sign of the remainder, the parity): the split names the
  rule of that one rounding (a register that keeps `s` unrounded, a different last operation);
- `P!D38` equals `other_neighbour` while the other rows are `ties_to_even`: the sum `v + s` is not formed from the
  stored `s` and the cells give its true remainder at the tie.

## val4-hyp-02: certain outcomes and the edges of the support

Rows 2-13 have draws = population (so x = successes with probability 1). Five cached Gate A points of that shape are
exactly 1 (`HYPGEOMDIST(26,42,26,42)`, `(37,59,37,59)`, `(4,51,4,51)`, `(7,9,7,9)`, `(27,54,27,54)`), while the
worksheet factor product is 0.9999999999999991, 0.9999999999999981, 0.9999999999999999, 1 and 1 on them. Rows 14-16
have successes = population; rows 17-19 sit beside those shapes ((25,41,26,42) and (26,41,26,42) are one draw short of the
population, (25,42,25,42) is one more certain shape); rows 20-28 put x on the smallest or largest possible
count, where one factor is the k = 0 or k = n closed form. Columns: E `HYPGEOMDIST`, F `HYPGEOM.DIST(..,FALSE)`,
G `HYPGEOM.DIST(..,TRUE)`, H p, I, J, K the factors, L = (I/K)*J.

Outcomes:
- E = F = 1 on rows 2-13 with L <> 1 on some: a certain outcome is returned as 1 before any factor is formed (the
  rule the engine change assumes for draws = population);
- E = L on rows 2-13: no shortcut, and the five cached 1s come from another form of the k = n factor inside
  HYPGEOMDIST (then I, J, K against E show which);
- rows 20-28, E against L: whether the edge factors inside HYPGEOMDIST are the worksheet's closed forms
  `EXP(n LN(p))` and `EXP(n ln(1-p))`.

## How to run

As val3: each case its own workbook, `rustsheet-oracle fill -b excel-graph --suite probes --case
probes/val4-hyp-01.case.json` (and `-02`). Check `Precision!B2:B4` = 0.9999999999999998, 1, 1.0000000000000002 and
`Precision!C2:C4` = -1, 0, 1 before reading anything else. Compare binary64 bits offline; no in-sheet residuals are
used.
