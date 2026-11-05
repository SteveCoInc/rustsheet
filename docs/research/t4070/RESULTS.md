# T-4070: bpser's two split items from T-3886 (negative results: both are GAMMALN's last digits)

Both items are recorded here and in docs/excel-semantics/statistics.md ("Incomplete beta bpser"). Neither needs a bpser
change. Every cell they cover where the research model matches Excel and the engine does not traces to the engine's
GAMMALN at the shape arguments, which T-2122 owns.

No harness was needed: the deleted T-2464 harness only supplied the engine's value and its GAMMALN, and both come from
`rustsheet-eval` and the bindings today. `docs/research/t2464/excelmath.py` `excel` is moved to today's
`rustsheet.Workbook` API (it still called the removed `rustsheet.create`), which `bratio_excel.fetch_missing` uses.

## Setup
- A venv with mpmath and numpy, and the bindings built into it: `maturin develop --release` in `bindings/python`.
- Item 2's `pre` and `post` reports come from worktrees at 1d3ea444 (T-2505's parent) and 81c3d27e (T-2505, #1991),
  each built with its own target dir: `cargo build --release -p rustsheet_eval`, then
  `rustsheet-eval --root <worktree> run --suite probes-numeric`. `now` is origin/main 724319d4.

## 1. The b0 >= 8 shortfall on t2464-r2-bpser-small-shapes (`l_diff.py`, `l_diff.out`)
- The cells are the 2,868 w_bpser cells with max(a, b) >= 8 (2,081 direct, 787 after bratio's swap), T-2977's set.
- Scores (ex/u4/w15): model GL53/glalgdiv 1,489/2,840/2,861, engine 1,484/2,842/2,859. These are T-2977's figures.
- Engine and model differ on 145 cells. On every one, Excel's probed GAMMALN(a0) differs from the engine's: 45
  distinct a0, all on file in t2464-r2-bpser-small-gl.
- Control: the model given the engine's GAMMALN equals the engine on 2,868 of 2,868. So L's form is already the
  model's, and the engine's algdiv in doubles never differs from the model's x87-stored algdiv on these cells.
- The two cells where only the model is within 15 digits are A9491 and A9492 (a = 8.789, b = 0.186). On both, the
  engine's GAMMALN(0.186) is 1 ulp above Excel's.

## 2. T-2505's 987 moved probes-numeric cells (`moved_cells.py`, `moved_cells.out`)
- The pre/post diff finds 987 cells that were bit-exact on the Cephes kernel and off after T-2505. That is T-2847's
  figure, so the reconstruction is exact.
- Today:

| class | cells |
|---|---|
| bit-exact again (later tasks) | 632 |
| model bit-exact, engine off ("port's" by T-2847's rule) | 196 |
| both off ("model's too"; 29 with engine != model) | 141 |
| not a literal BETA.DIST cdf (cell references, BETA.DIST of a reference, a ratio) | 18 |

- Of the 355 still off, 269 are 1 ulp off, 67 are 2, 15 are 3 or 4 and 4 are 5 or 6.
- The 196 "port's" cells are all GAMMALN's last digits, not the port:
  - With the engine's GAMMALN in L, the model gives the engine's exact value on 178.
  - The other 18 are at min(a, b) < 1, b0 < 8, where the engine has T-3879's split z and r2-bpser-small's model does
    not. Under r7's split form (`r7-bpser-nondouble-z/forms.py` KEY), the engine's GAMMALN gives the engine's value
    on 18 of 18, and Excel's GAMMALN gives Excel's on 18 of 18.
- By case, the 196 are t2464-w2f-xruns-2 88, t2464-r2-bpser-small-shapes 76, -runs 27, t2464-w2f-unit-shape 3 and
  t2464-r2-bratio-x29 2.

## What would move these cells
Excel's GAMMALN to the bit at shapes below 2 (T-2122). r2-bpser-small found the engine's model exact on 1,738 of
2,585 GAMMALN cells below 1 in t2464-r2-bpser-small-gl (docs/research/t2464/r5-bpser-lngamma/RESULTS.md, which quotes
it). That is an older engine: today's eval passes 4,471 of that case's 5,843 cells, over all arguments. No bpser, L or algdiv change is indicated.
