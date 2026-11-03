# T-3873 r7-bpser-nondouble-z: Excel's E for a non-double z = fl(a0 LN x) - L in bpser

Round 7, 2026-11-03. Question (T-3769, r6-bpser-xlinked RESULTS.md): GL53's bpser is bit-exact wherever z = alx - L
is a double, and misses where z needs 54-55 bits. How does Excel's bpser treat that z?

- Python: `bindings/python/.venv/bin/python` (`make py-bindings`, then `uv pip install mpmath numpy scipy` into it).
  Run the scripts from this directory. `c7.py` imports r6-bpser-xlinked's `common.py` (and through it r5, r2 and
  r2-bratio). `lg.py` imports r6-bpser-gpin's `gpin.py`. Nothing is copied. No new probe: every cell scored here is an
  existing excel-graph fill.
- Scores are bit-exact / within 4 ulp / within 15 significant digits of Excel's value.

## Answer

**Excel splits z.** With zh = z rounded to the nearest double (ties to even) and zl = z - zh (exact, and a double),

```
I = fl( fl(EXP(zh) · s) · fl(1 + zl) )      every operation a stored double; E = Excel's EXP of a double
```

Where z is a double, zl = 0 and this is GL53's I = fl(EXP(z) s) unchanged. The order is fixed by the data: s first,
then the (1 + zl) factor. fl(1 + zl) equals Excel's EXP(zl) on every zl the split leaves (|zl| <= 2^-53 here: Excel's
EXP(2^-53) is 1.0 by double rounding, EXP(-2^-53) is 1 - 2^-53), so BETA.DIST cells cannot tell
fl(EXP(zh) s)·EXP(zl) from fl(EXP(zh) s)·fl(1 + zl). Either is Excel's form.

| set | cells | double z: GL53 / split | non-double z: GL53 (z64) / split |
|---|---|---|---|
| r6 tune (t3769-r6-bpser-beta, b0 2.5, 6.25) | 3,072 | 1,524 / 1,524 of 1,526 | 808 / **1,546 of 1,546** |
| r6 hold-out (b0 4.75, fixed before the fill) | 1,536 | 397 / 397 of 397 | 560 / **1,139 of 1,139** |
| r5 (t3570-r5-bpser-lngamma-beta) | 1,920 | 1,143 / 1,143 of 1,143 | 360 / **777 of 777** |
| T-3771 (t3570-r5-bpser-lg, a0 0.15-0.85) tune | 2,064 | 769 / 769 of 770 | 661 / 1,292 of 1,294 |
| T-3771 hold-out (`pin.hold`) | 1,968 | 849 / 849 of 849 | 585 / 1,118 of 1,119 |
| w2f (0.5, 6.25) xruns-1 and absweep, GAMMALN measured | 1,089 | 10 / 10 of 10 | 521 / **1,079 of 1,079** |

Exact / 4 ulp / 15 digits for the split model
(`misses.out`): r6 tune 3,070 / 3,072 / 3,072, hold-out 1,536 / 1,536 / 1,536, r5 1,920 / 1,920 / 1,920.

The two r6 misses are the double-z cells at (6.25, 0.7), x = 0.0394 and 0.0409, both +1 ulp. r6's `esplit.out`
shows that moving s down one ulp fixes both, so they are an s-side residual, not z's.

**The same split holds in the b0 <= 1 sub-branch** (`shapes.out`, r2-bpser-small's probes, cells whose only series
call is bpser on the cell's own (a, b, x)):

| r2 probe, sub-branch | double z: GL53 / split | non-double z: GL53 / split | split with TOMS's gamln1 g |
|---|---|---|---|
| shapes (2,000 shapes, never tuned on), b0 <= 1 | 692 / 692 of 693 | 861 / **1,519 of 1,519** | 536 + 1,223 |
| runs, b0 <= 1 | 478 / 478 of 478 | 888 / **1,569 of 1,570** | 350 + 1,186 |
| shapes, 1 < b0 < 8 | 767 / 767 of 770 | 1,458 / 2,639 of 2,645 | 640 + 2,149 |
| runs, 1 < b0 < 8 | 1,519 / 1,519 of 1,519 | 1,601 / 2,041 of 2,065 | 1,519 + 2,041 |

This is r2-bpser-small's "b < 1 shapes top out at 55-84% for any constant L" ceiling (its section 4, and the
statistics.md entry's Open bullet). The unidentified stored intermediate was the split of z.

## 1. How it was found

- `implied.py` (`implied.out`): for every non-double-z cell, the doubles E with fl(E s) = Excel's I. On many cells
  no double E within 8 ulp works, and on others Excel's E would sit below EXP(z rounded down). So no E alone (no
  rounding of z, no EXP of a perturbed z) explains Excel. The low part of z must reach I some other way.
- `forms.py` (`forms_tune.out`, `forms_hold.out`, `forms_r5.out`): 4 ways to round zh (nearest-even, down, up,
  toward zero) times 14 ways to carry zl. Only nearest-even with the factor applied last, as a stored double, is
  exact everywhere. On the tune non-double cells, against its 1,546:
  - extended (1 + zl), or P + P·zl with P = fl(E s), stored or extended: 999;
  - fl(E·fl(1 + zl))·s: 1,320; E·fl(s·fl(1 + zl)): 1,277; the extended products (E f) s and (E s) f: 1,275 and 1,278;
  - E + E·zl, stored or extended, then times s: 803 and 853; E·(s + s zl): 824;
  - zh rounded down: at most 1,111; up or toward zero: at most 1,192.
- The form was chosen on the tune shapes only. The r6 hold-out, r5, T-3771's probe, the old w2f run and r2's b0 <= 1
  cells were scored after, with no change to the form.

## 2. Positive controls

- Double-z cells reproduce r6's GL53 exactly: 1,524 of 1,526 tune and 397 of 397 hold-out, the same cells as
  `repr_r6.out`. r6's `controls.out` (LN 511 of 511, a0·LN x 1,533 of 1,533, EXP 10,848 of 10,848) still holds;
  nothing here changes those pieces.
- `nonrepr.py` of r6 re-run from this tree gives `nonrepr.out` byte for byte.

## 3. The other open items of T-3873

- **r2-bpser-small's old w2f (0.5, 6.25) shapes** (`w2f.py`, `w2f.out`). With Excel's measured GAMMALN at a0, b0 and
  fl(a0 + b0), the split model is exact on all 1,089 cells: xruns-1's 1,024 (GL53 z64 494) and absweep's 65 (37).
  Absweep's 23 cells whose GAMMALN(a0) was never measured get 8 (GL53 5). Those misses are the GAMMALN model's
  (T-3042), not z's. r2's "the old w2f shapes prefer g1ln53" reading came from the unsplit z.
- **GL53 against the gamln1 form at small a0 and b0 <= 1** (`shapes.out`, last column). With the split, TOMS's
  g = fl(gamln1(a0) - LN a0) loses at b0 <= 1: shapes 1,759 against 2,211 of 2,212, runs 1,536 against 2,047 of 2,048.
  On T-3771's probe the a0 = 0.15 center is exact with GL (67 + 197 hold-out, 76 + 236 tune). So g is Excel's
  GAMMALN(a0) at b0 <= 1 too.
- **b0 >= 8** (`shapes.out`). The split does not help there yet. L is unsolved (T-3569): GL53 is exact on 0 of 235
  double-z cells of the runs. Once L is right, the split should apply as it does below 8. On 870 of the shapes and runs
  b0 >= 8 cells the kernel's algdiv L differs from `MODEL.lnb_w3`, so they are not scored.

## 4. Method notes

- No fitted constant (decision 0056): the forms are fixed structures, scored as wholes. The structural choice (which
  rounding of zh, where the factor goes) was made on r6's tune shapes, then frozen. The hold-out (r6 b0 4.75, fixed in
  r6's `make_probe.py` before its fill) and four independent probes (r5, T-3771's, w2f, r2) are the fresh sweep. All
  were scored after the freeze.
- Not a Gate A envelope question: the result is a bit-exact form, with no accuracy trade-off.
- Mechanism: the data fix the arithmetic, not the source. A bpser that holds z as a double pair (zh, zl) and applies
  EXP(zl) or (1 + zl) to the product would produce it.

## 5. Follow-ups

- **T-3879** (functions-stat, engine): use the split in `bratio/bpser.rs` where L is GL53's double
  (min(a, b) < 1, b0 < 8).
- b0 >= 8 L stays T-3569. The 30 remaining non-double misses at 1 < b0 < 8 on r2's runs and shapes (24 and 6), and the 2
  s-side misses at r6 (6.25, 0.7), are below the size of a task; they are recorded here.

## 6. Files

- `c7.py`: loaders (`cells`, `cell`) and `e_ok` (the E doubles consistent with Excel's I).
- `implied.py`, `forms.py`, `misses.py`, `w2f.py`, `lg.py`, `shapes.py`. Each `X.py` writes `X.out`, and `forms.py`
  writes `forms_<set>.out` (`forms.py tune|hold|r5`).
