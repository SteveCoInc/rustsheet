# T-3769 r6-bpser-xlinked: bpser's residual misses at a0 < 1 < b0 < 8

Round 6, 2026-11-02. Question (T-3570, r5-bpser-lngamma RESULTS.md section 3): GL53's bpser misses 417 of 1,920 cells
on t3570-r5-bpser-lngamma-beta. At a0 near 0.3 the misses follow x. At 0.5 and 0.7 they partly follow a0. Which stored
intermediate carries each kind?

- Python: `bindings/python/.venv/bin/python` (`make py-bindings`, then `uv pip install mpmath numpy scipy` into it). Run
  the scripts from this directory. `common.py` imports r5-bpser-lngamma's `make_probe`/`score_forms`, and through them
  r2-bpser-small's `lib.py` and r2-bratio's `bratio_excel.py`. `lcover_r6.py` imports r2-bpser-small's `lscan.py`.
  Nothing is copied.
- Scores are bit-exact / within 4 ulp / within 15 significant digits.
- GL53 is the r5 model: L = fl(GL(a0) + fl(GL(b0) - GL(fl(a0 + b0)))) with Excel's measured GAMMALN,
  z = fl64(fl(a0 LN x) - L) extended, E = EXP(z) (Excel's EXP model on the extended argument), w3's stored series s,
  and I = fl(E s).

## Answer

**Both residuals are one effect: they occur only where z = alx - L is not a double.** Wherever z is a double, GL53 is
bit-exact:

| probe | z is a double | z is not a double |
|---|---|---|
| r5 (a0 by ulps, 1,920 cells; `repr_r5.out`) | 0 misses / 1,143 | 417 / 777 |
| r6 (x at fixed shapes, 4,608 cells; `repr_r6.out`) | 2 / 1,923 | 1,317 / 2,685 |

On those cells E is Excel's own worksheet EXP of the same double, measured cell by cell (`esplit.out`: 1,921 of 1,923
cells exact with the measured E). So s, alx = fl(a0 LN x), L and the product fl(E s) are Excel's. The missing piece is
how bpser's EXP treats an argument that needs more than 53 bits. With alx near -1 and L near 1-2, z = alx - L is exact
in 54-55 bits. Nearly every non-double z sits exactly halfway between two doubles (`nonrepr.out`).

- **x-linked (a0 near 0.3).** At a fixed shape L is constant, and the cell misses when alx's last bit leaves z
  between doubles. Every such miss has the model 1 ulp above Excel.
- **a0-linked (0.5, 0.7).** z's low bits also depend on L's last bits, which move with a0. When L puts z between
  doubles at both x, both x miss together. On r5, all 93 a0 where both x miss have a non-double z at both x.
- **This is not an L error.** On every r6 shape at a0 0.3, and at (2.5, 0.7) and (6.25, 0.7), the best constant L along
  x is GL53's own L (`lcover_r6.out`). At a0 = 0.5 every z is non-double under GL53 (512 of 512 per shape), so
  `lcover_r6.py` cannot pin L there.

Not solved: Excel's E for a non-double z. It is not a double rounding of z. Per tie cell, Excel's I is matched by EXP of
the extended z on some cells, by EXP of z rounded down on others, and by neither on the rest (`nonrepr.out`).

## 1. Probes (excel-graph, 2026-11-02, 0 failures)

`make_probe.py --write` writes the beta and aux cases. `make_probe.py exp --write` writes the exp case, once aux is
filled, since it needs Excel's GAMMALN for L.

| case | cells | what |
|---|---|---|
| `t3769-r6-bpser-beta` | 4,608 | `BETA.DIST(x, a0, b0, TRUE)`, a0 in {0.3, 0.5, 0.7}; tune b0 2.5 and 6.25, hold-out b0 4.75 (split fixed before the fill). Per shape: 128 consecutive doubles up from 0.0371 and x = 0.0100 + 0.0001 k, k < 384. Every cell is a direct w_bpser cell ((x b0)^a0 <= 0.7 at (0.3, 6.25) needs x <= 0.0488). |
| `t3769-r6-bpser-aux` | 2,059 | `=LN(x)` at all 511 x, `=GAMMALN` at the 15 a0, b0 and fl(a0 + b0), and `=a0*LN(x)` |
| `t3769-r6-bpser-exp` | 10,848 | `=EXP(z)` at the stored z of every beta cell and its ±1-ulp neighbours |

Doubles are written exactly. `snum` extends w2-forensics' `num` to negative values: its encoder assumes v > 0. A
first fill of the exp case used the unextended encoder. Excel cut its 16-digit literals to 15 digits, and the EXP
control failed (954 of 10,848). The case was rebuilt and refilled; the committed sidecar is the second fill.

**Refill (decision 0112).** t3570-r5-bpser-lngamma-beta was refilled from excel-graph on 2026-11-02 with
`rustsheet-oracle fill --force`. All 1,920 cells were identical to the 2026-11-01 fill, so r5's 1-2 ulp bits
reproduce. The committed sidecar was left as it was.

## 2. Positive controls (`controls.out`, `esplit.out`)

- LN model = Excel `=LN(x)` on 511 of 511.
- `=a0*LN(x)` = the stored x87 product on 1,533 of 1,533 (the SSE double product agrees on all of them too).
- EXP model = Excel `=EXP(z)` on 10,848 of 10,848.

## 3. Negatives, with counts

- **z stored to the nearest double (z53).** It won on r5's data: 1,628 against 1,503 (`variants_r5.out`). On the fresh
  probe it loses: tune 2,072 against 2,332, hold-out 670 against 957, and 0 of 512 at (4.75, 0.7) (`score_z.out`).
  This is a selection effect, so r5's model (z extended) stays.
- **Other roundings of z** (`zround.out`): rounding down scores tune 2,403 and hold-out 1,055 against z64's 2,332
  and 957. Down wins at a0 0.3 and 0.5 but loses at (2.5, 0.7), 261 against 406. Rounding up: 1,542 and 403. No rule
  fits every shape.
- **Compositions of z or E** from the same measured pieces (`zcomp.out`): L kept extended, (g + gb) - gab grouping,
  sequential ((alx - g) - gb) + gab, extended or stored, EXP(alx)·EXP(-L), EXP(alx)/EXP(L). None beats GL53 on the
  tune shapes. EXP(alx)·EXP(-L) gains at a0 0.5 and at (4.75, 0.7) but loses 90-120 cells at every a0 = 0.3 shape.
- **Product splits** (`product.out`): E·ia + E·sp, E/a + E·sp, (x^a·s)·B^-1, x^a·(s·B^-1), (x^a·B^-1)·s, (x^a/B)·s and
  (x^a·s)/B are all below E·s on 7 of 9 shapes. The two exceptions, (4.75, 0.5) and (4.75, 0.7), are shapes where
  every z is non-double.
- **Series** (r5 data with z53; `series_z53.out`, `stop_z53.out`, `acc_r5.out`): extended accumulation, a correctly
  rounded s, TOMS's (1 + a Σ) and the cephes and fwd0 orders all lose (758-1,583 against 1,628). Stopping at |w| <= eps/a
  or eps·s, for eps in {2^-52, 1e-15, 1e-14}, loses or ties at 2^-53. The model's s is 1 ulp off the correctly rounded
  series on many exact cells too, so s's own error does not predict a miss.
- **lnΓ(a0) other than GAMMALN** (`gscan.out`, `repr_r6.out`). At exactly a0 = 0.5, g = GAMMALN(0.5) + 1 ulp
  (= TOMS's fl(gamln1(a0) - LN a0)) scores higher on all three shapes: 395, 355 and 336 against 284, 223 and 248. It
  is still not Excel's L. With it z is a double on 1,389 of 1,536 cells, so E is Excel's own EXP, and it misses 47, 157
  and 163 of those cells. A right L leaves no misses there. It wins only because it moves z onto the double grid,
  away from the unsolved EXP piece.
  - Excel's GAMMALN is equally accurate at 0.5 and its neighbours (`gl_half.out`: -0.05 ulp at 0.5, mean -0.14 on 580
    neighbours), so this is no worksheet special case.
  - On r5's a0 near 0.5 by single ulps, g = GAMMALN still wins: 237 and 235 against 171 and 139 (`gshift_r5.out`).
- Where the misses fall (`misspos.out`, `frac.out`): at a0 0.3 they are mostly at I >= 0.5 (larger x). The unrounded
  E·s of a miss sits anywhere in I's ulp, rarely 0.2-0.4 above a double, so Excel's E there is lower than the model's
  by a variable 0.05-0.6 ulp of I. That fits an E different from EXP(z64), not a product near-tie.

## 4. Method notes

- Fitting: there is no fitted constant. The conclusions compare fixed forms, and the double/non-double split is a
  property of each cell's own arithmetic. Tune is b0 2.5 and 6.25, and the hold-out is b0 4.75, fixed in
  `make_probe.py` before the fill. The double-z result holds on the hold-out (0 of 397 misses at (4.75, 0.3)) and on
  r5's earlier, independent probe (0 of 1,143).
- Section 3's z53 negative is an example of why the fresh sweep matters: the r5-only reading reversed on new data.

## 5. Follow-ups

- **T-3771** (open, research) already asks whether the bpser ceiling sits in LN or in EXP's argument. This round
  answers it for 1 < b0 < 8 at a0 0.3, 0.5 and 0.7: LN is exact, and the ceiling is EXP of a non-double z. The new
  follow-up T-3873 takes the EXP piece.
- No engine follow-up: no piece is solved for the engine. GL53 with an extended z stays the engine's model
  (`functions/statistical/bratio/bpser.rs`).
- Not done here: from T-3570's review, re-scoring r2-bpser-small's old w2f (0.5, 6.25) shapes with measured GAMMALN,
  and the GL53-vs-gamln1 comparison at small a0, b0 <= 1 and b0 >= 8. Both move to T-3873, since any re-score must
  first split the cells by double z.

## 6. Files

- `common.py`: the loaders, `pieces()` (GL53's stored intermediates) and `step()`.
- `make_probe.py`: writes the three cases and their `*.meta.json` (cell → kind, x, a0, b0).
- On r5's probe: `fix_r5.py`, `acc_r5.py`, `variants_r5.py`, `resid_z53.py`, `series_z53.py`, `stop_z53.py`,
  `gshift_r5.py`, `repr_r5.py`.
- On r6's probe: `controls.py`, `score_z.py`, `lcover_r6.py`, `lpieces.py`, `gscan.py`, `misspos.py`, `frac.py`,
  `product.py`, `zcomp.py`, `esplit.py`, `zround.py`, `nonrepr.py`, `repr_r6.py`, `gl_half.py`.
- Each `X.py` writes `X.out`.
