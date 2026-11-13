# Probe request: F.INV.RT across the saturation band (T-4894, poc-gateb-cc)

One case, 66 checked cells (60 formulas and the Precision canary), excel-graph, raw values:
`eval/suites/probes/cases/t4894-f-inv-rt-saturation-band.case.json`. It is on branch `gateb/f-flipped-saturation`
(CR-321681168) and copied here at its suite path.

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t4894-f-inv-rt-saturation-band.case.json
```

`Precision!B2:B4` must read 0.9999999999999998, 1, 1.0000000000000002 and `Precision!C2:C4` -1, 0, 1.

## What it asks

`F.INV.RT(p,df1,1)` with literal arguments at df1 = 1, 2, 3, 9 and fifteen p from 1E-110 to 1E-190. No Excel cell
on file lies between p = 1E-100 and p = 1E-200 at df2 = 1. The engine has three regimes there
(`docs/research/t4894/saturation-band-predictions.tsv`, hypotheses, never expected values):

| p | Engine today |
|---|---|
| 1E-110 to about 5E-155 | the quantile, a normal double |
| about 5E-155 to 1E-162 | `#NUM!`, or for df1 of 3 and 9 the end of the search bracket near DBL_MAX/df1, alternating with p |
| below about 1E-162 | the search's saturated stop, about (1/df1)/2.7055E-151 (CR-321681168) |

The middle regime is a known defect (review of CR-321681168 round 1, reviewer 1 B1): the true quantile is past the
doubles there, and the engine returns a search bracket's end. Excel's values decide what it should return: `#NUM!`
across the band, the saturated stop from higher p than the engine's, or something else.

Not urgent: it unblocks no CR. It decides the next piece of T-4894.
