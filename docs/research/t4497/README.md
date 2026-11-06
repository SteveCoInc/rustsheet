# T-4497: the a = 1 cut of F.TEST and the df 2 F quantiles

Data for T-4498, which fits `A1_EXPM1_MAX_X` (incomplete_beta.rs) and the kernels below it. No engine
change here.

## The sweep

- `probes-numeric/t4497-f-a1-cut-sweep` (round 1, 480 cells) and `probes-numeric/t4497-f-a1-cut-dense`
  (round 2, 180 cells), both oracle-filled by excel-graph on 2026-11-06. `mkcase.py` writes both cases
  and `points.tsv`.
- Each case has 3 columns of calls. Column A is `F.TEST({-1,0,1},{...})`: the a side is a 3-value
  sample (variance 1, a = 1), and the other sample is ±c pairs, df 2b, b from 1.5 to 50. Column B is
  `F.INV(p,2,df2)` and column C is `F.INV.RT(p,df1,2)`, with df from 1 to 50 (never 2) and p < 0.5.
  Then the searched tail is the unswapped a = 1 one: `fisher.rs` `f_quantile_tail` →
  `unoriented_shape_one_tail`.
- Round 1 has 160 calls a column, with the a = 1 tail x log-uniform over 4E-4..0.2. Round 2 has 60 a
  column, with x log-uniform over 0.095..0.105, around the cut round 1 showed.
- `points.tsv` has one row per cell: the x of the call (in doubles, as the engine forms it), the
  shape b, a 50-digit math value of the whole call from the decimal literals (mpmath, 60 digits
  working), and the hold-out flag.
- **Split, fixed before any scoring** (decision 0188 R4.1). A cell is held out when sha256 of
  `case!cell` mod 100 is below 30. `mkcase.py` `holdout` wrote the flag before `score.py` ran on
  either round. That gives 194 of 660 cells (29.4%) held out: 133 in round 1 and 61 in round 2. The
  margin for T-4498 is 5 points: a candidate's hold-out bit-exact rate should be within 5 points of
  its training rate.
- The Gate A tooling (`rustsheet-accuracy math`) writes `.math.json` sidecars for the cells it
  covers. F.TEST and F.INV.RT have no definition there, so `points.tsv` carries the math values of
  every cell.

## Scores (`score.py`, output in `score.out`)

`score.py` reads `rustsheet-eval show --all` output from a scratch build. In that build,
`unoriented_shape_one_tail` takes its form from an environment variable (never committed). There are
three uniform forms:

- `libm`: -expm1(b ln1p(-x)) by libm;
- `cancel`: 1 - exp(b ln(1 - x));
- `excel`: Excel's 1 - e^y (`x87_exp::excel_one_minus_exp`, Kahan's expm1) of y = b times its
  ln(1 - x) (`x87_log1p::excel_ln_one_minus`, the stored atanh series).

Under a cut c, a cell with x < c takes the form below the cut, and every other cell takes `cancel`.
`bit` means bit-exact, `<=4ulp` within 4 ulp, `15d` the same to 15 significant digits, and `off`
visibly off.

| form | training (466) | hold-out (194) |
|---|---|---|
| today: cut 0.1, libm below | 146 bit, 436 <=4ulp, 461 15d, 5 off | 59 bit, 180 <=4ulp, 190 15d, 4 off |
| cut the data suggests (best training plateau 0.099798..0.10004, both kernels), libm below | 146 bit, 436, 461, 5 off | 59 bit, 180, 190, 4 off |
| cut 0.1, Excel kernels below | 192 bit, 437 <=4ulp, 462 15d, 4 off | 88 bit, 179 <=4ulp, 190 15d, 4 off |
| libm everywhere (no cut) | 119 bit, 450, 464, 2 off | 48 bit, 183, 191, 3 off |
| Excel kernels everywhere (no cut) | 171 bit, 448, 464, 2 off | 77 bit, 182, 192, 2 off |

F.TEST separates the forms cleanly. The quantiles do not:

| function | cut 0.1, libm below (train / hold) | cut 0.1, Excel kernels below (train / hold) |
|---|---|---|
| F.TEST (220) | 100/149, 42/71 bit; 0 off | **149/149, 71/71 bit**; 0 off |
| F.INV (220) | 24/156, 11/64 bit; 4 off | 22/156, 10/64 bit; 3 off |
| F.INV.RT (220) | 22/161, 6/59 bit; 5 off | 21/161, 7/59 bit; 5 off |

## Findings

1. **The cut is at 0.1.** Every F.TEST cell from x = 0.100089 up is bit-exact only with `cancel`
   (or with all forms). Every F.TEST cell up to x = 0.0996048 is bit-exact with `excel`, and none of
   those with `cancel` alone. The nearest pair is dense A15 (x 0.0996048, `libm,excel`) and dense A12
   (x 0.100089, `cancel`). The training scan's best plateau is 0.099798..0.10004, and 0.1 lies in it.
   So today's constant stands: no cut the data suggests scores better than 0.1.
2. **Below the cut F.TEST uses Excel's own expm1 and log1p, not libm's.** With them all 220 F.TEST
   cells are bit-exact, 71 of 71 held out. With libm's, the engine's form today, 142 are bit-exact.
   This is the same pair of kernels `shape_one_tails` already takes for the oriented tails (T-3170),
   so T-4498 can share that one definition (directive 3).
3. **The df 2 quantiles do not place the cut.** Only about 14% of their cells are bit-exact under any
   form. The search stops a few ulps from the crossing, and changing the tail's last bits moves that
   stop in no consistent direction: a cell can be bit-exact under one form and visibly off under the
   next (round 1 B157, x 0.0022, is bit-exact only under `cancel`). Per form, over the 440 quantile
   cells:

   | x range | cells | libm | cancel | excel |
   |---|---|---|---|---|
   | below 0.1 | 334 | 52 bit, 1 off | 15 bit, 168 off | 49 bit, 0 off |
   | from 0.1 up | 106 | 15 bit, 4 off | 11 bit, 8 off | 19 bit, 4 off |

   Below 0.1 the quantiles agree with F.TEST: the cancelling form is clearly wrong there. From 0.1 up
   the quantile cells lean slightly against the cut: `cancel` is the worst form, by 8 off against 4.
   That is noise-sized on 106 cells, but T-4498 should check whether the search's tail cuts at 0.1 too
   or keeps the uncancelled form. Its fresh sweep is the place for that.
4. **Visibly-off cells.** Today 9 quantile cells are visibly off: round 1 B157, C52 and C123; round 2
   B9, B41, B58, C9, C27 and C44 (values in `score.out`). Under a cut of 0.1 with Excel's kernels
   below it, 8 are off. Under Excel's kernels with no cut, 4 are off. Round 2 B58 is off under every
   form. T-3485, the owner of the F quantiles' visibly-off cells, now lists all 9.

## Reproduce

```
cd tools && uv run python ../docs/research/t4497/mkcase.py --dense 0.095 0.105   # cases + points.tsv
# scratch build: in fisher.rs unoriented_shape_one_tail, select libm / cancel / excel by T4497_MODE
for m in "" libm cancel excel; do for c in sweep dense; do
  T4497_MODE=$m <scratch>/rustsheet-eval show probes-numeric/t4497-f-a1-cut-$c.case.json --all \
    > show/${m:-today}.t4497-f-a1-cut-$c.show; done; done
python3 docs/research/t4497/score.py show > docs/research/t4497/score.out
```
