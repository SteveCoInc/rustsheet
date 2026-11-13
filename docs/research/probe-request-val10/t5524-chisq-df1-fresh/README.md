# Probe request: fresh cells for CHISQ.INV.RT and CHIINV at one degree of freedom (decision 0188 R4.2; T-5524, poc-gateb-cc)

One case, 515 checked cells, excel-graph, raw values. It has grown three times, each time before any fill: 294 cells
at first, 342 at 02:2xZ on 2026-11-12 (sheet Wide), 503 at 03:1xZ (sheets WideHalf, Low, LowCuts and Twins, review
round 1 of CR-321929035), 515 at 06:2xZ (review round 2: Wide redrawn so that each decade has both sides of the
cut, Twins re-chosen, sheet LowNames added). The other ten sheets are byte-identical to the 503-cell state:
`eval/suites/probes-numeric/cases/t5524-chisq-df1-complement-fresh.case.json`, copied here at its suite path. It is
not on main yet (CR-321929035).

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric --case probes-numeric/t5524-chisq-df1-complement-fresh.case.json
```

What comes back: `eval/suites/probes-numeric/cases/t5524-chisq-df1-complement-fresh.oracle.json`.

## Why

CHIINV fails the expert tier gate (decision 0355): 9 cells outside the envelope of 681, 6 allowed. They are the
cells where Excel is "one cdf step" from the quantile of `1 - p` (T-3336), which T-3337 time-boxed with "reopen
only with a new hypothesis". There is one now, and it also gives T-3858's `#NUM!` edge, which the engine carries
as a fitted constant.

Excel's CHISQ.INV.RT and CHIINV at one degree of freedom are the lower quantile of `(0.5 - p) + 0.5` formed with
the worksheet's own twice-rounded `-` and `+`, not of `1 - p`, and `#NUM!` where that complement is 1. On the 518
df-1 cells with a literal p in the probe suites and Gate A's tables, the exact quantile of that complement is
within Excel's 15 digits on 346 of 347 numbers and has no value on all 171 of Excel's `#NUM!`; of `1 - p`, 298 and
6. It is a structural rule found by reading Excel's cells, so decision 0188 R4.2 owes it Excel values drawn after
it was frozen. The engine change is built (branch `gateb/chisq-df1-complement`) and waits for this fill.

## What is in it

k is p / 2^-53. Sheets, none chosen by Excel's answer:

| Sheet | Cells | What it is |
|---|---:|---|
| Plain | 80 | p log-uniform over [1E-12, 0.25) |
| Quarter | 64 | k an odd n plus or minus (1/4 + d), d = +-0.02 and +-0.05: both sides of the cuts the rule has |
| Half | 32 | k = n + 1/2 +- 0.03: both sides of the cut `1 - p` has |
| Register | 32 | k within 3 * 2^-14 of a quarter cut, where the 64-bit register decides |
| Edge | 24 | the `#NUM!` edge 3 * 2^-55 - 2^-66 and its neighbours, as literals and as sums of powers of two |
| Upper | 20 | p from 0.25 to 1 - 1E-12, where every form of the complement is the same double |
| Names | 42 | CHIINV, and degrees of freedom 1.5 and 1.99 |
| Wide | 48 | the quarter cuts at larger p, twelve in each decade from 1E-7 to 1E-3 (the first draw's acceptance test kept only p below 2E-7 on Quarter, Half and Register) |
| WideHalf | 24 | the half cut at those larger p |
| Low | 40 | p between 2^-53 and 1E-12, ten in each decade |
| LowCuts | 48 | the quarter and half cuts there, n from 3 to 9007 |
| Twins | 49 | `CHISQ.INV(1-n*2^-53,1)` beside six rows of each sheet: the rule's complement written out, which must give the row's value bit for bit |
| LowNames | 12 | CHIINV on six rows of Low and six of LowCuts |

Every p is a literal of at most 15 significant digits, or on Edge a sum of powers of two (`3*2^-55-2^-66`), which
must stay a formula: typed as a decimal it would be cut to 15 digits. The Twins arguments (`1-n*2^-53`) must stay
formulas for the same reason.

## Frozen before the fill

`docs/research/t5524/fresh-predictions.tsv` (per cell: p, k, the complement under `1 - p`, under plain doubles and
under the rule, and the exact quantile of each) and `fresh-engine.tsv` (the engine with the rule). The rule's
prediction differs from `1 - p` on 174 of the 466 CHISQ.INV.RT and CHIINV cells and from the plain-double form on 20, so the fill can refute it. The
read of the fill is fixed in `docs/research/t5524/README.md` on the branch.

No urgency beyond the next batch.
