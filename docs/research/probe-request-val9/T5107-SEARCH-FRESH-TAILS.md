# Probe request: the strata the first fresh probe leaves out (decision 0188 R4.2; Taskei P559210060, poc-gateb-cc)

One case, 287 checked cells (281 formulas and the Precision canary), excel-graph, raw values:
`eval/suites/probes/cases/t5107-search-fresh-tails.case.json`, copied here at its suite path; it is on branch
`gateb/search-fresh-probe` (CR-321714108).

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t5107-search-fresh-tails.case.json
```

`Precision!B2:B4` must read 0.9999999999999998, 1, 1.0000000000000002 and `Precision!C2:C4` -1, 0, 1. A canary
failure means stop. What comes back: `eval/suites/probes/cases/t5107-search-fresh-tails.oracle.json`.

## Why a second case

The reviewer of CR-321714108 found that the first fresh probe (`t5107-search-fresh`, in val9) cannot stand as the
full-domain sweep: it has no inverse probability above 0.981, none below 1E-300, and no small, large or extreme
shape. Its workbook is already with the oracle, so it is left as it is, and this case holds what it lacks. The
search change cannot land without both.

## What is in it

| Stratum (column A) | What | Sweep sheet W | Aimed sheet D |
|---|---|---:|---:|
| UT | BETA.INV, BETAINV, F.INV, F.INV.RT, FINV at p = 1 - q, q from 1E-15 to 1/2 | 30 | 15 |
| DN | the same at p from 2.22507385850721E-308 to 1E-300 | 30 | 15 |
| SS | BETA.INV, BETAINV with a shape from 1E-6 to 0.1 | 30 | 10 |
| LS | the inverses with a shape up to 1E10 or a df up to 1E10 | 30 | 15 |
| XS | BETA.INV with a shape from 1E-300 to 1E-6 or from 1E10 to 1E300 | 20 | 3 |
| CU | BETA.DIST cumulative at x = 1 - q | 30 | 8 |
| CH | BETA.DIST cumulative with a shape up to 1E10 | 30 | 15 |

Sheet W was chosen without reading either engine's value; sheet D is the first later formulas of each stratum on
which the engine before the change and the engine with it differ. `search-fresh-tails-predictions.tsv` (copied
here) holds both engines' values for every cell: hypotheses, never expected values.

Some cells will be `#NUM!` in Excel (extreme shapes, df above its limits): that is an answer, not a failure of
the fill.
