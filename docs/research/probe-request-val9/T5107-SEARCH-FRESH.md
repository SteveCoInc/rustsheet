# Probe request: fresh cells for the BETA.INV / F.INV search change (decision 0188 R4.2; Taskei P559210060, poc-gateb-cc)

One case, 526 checked cells (520 formulas and the Precision canary), excel-graph, raw values:
`eval/suites/probes/cases/t5107-search-fresh.case.json`, copied here at its suite path. It is not on main yet; it
will be on branch `gateb/f-search` with the change.

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t5107-search-fresh.case.json
```

In the workbook `Precision!B2:B4` must read 0.9999999999999998, 1, 1.0000000000000002 and `Precision!C2:C4` -1, 0, 1.
A canary failure means stop. What comes back: `eval/suites/probes/cases/t5107-search-fresh.oracle.json`.

## Why

The owner approved landing the T-5107 search candidate (BETA.INV, F.INV, F.INV.RT and FINV on the worksheet
kernels: bpser's split z for both shapes above 1, the identified density as bfrac's prefactor and as the search's
slope, the subnormal-residual stop, the twice-rounded step, the 516-evaluation cap). It is a structural model chosen
by scoring candidates on Excel cells, so decision 0188 R4.2 owes it Excel values drawn after it was frozen. The same
guard blocked CR-321684429 (LINEST) in review; this request is made before the CR so that it does not cost a round.

## What is in it

Every formula carries its own literals (15 significant digits at most). All 520 come from one seeded pool of 4,800
(`docs/research/t5107-fin-val4/fresh_pool.py` on the branch), 600 in each of eight groups, log-uniform over each
group's domain. No cell was read to shape the change.

| Sheet | Formulas | What it is | The two engines differ on |
|---|---:|---|---:|
| W | 320 | The sweep: the first 40 formulas of each group, taken without reading either engine's value | 36 |
| D | 200 | Aimed: the first 25 later formulas of each group on which the two engines differ | 200 |

Groups (column A): CS `BETA.DIST(..,TRUE)` at both shapes in (1, 8]; CL the same at shapes up to 1000; FC `F.DIST`
cumulative and `F.DIST.RT`; BN `BINOM.DIST` cumulative; BI `BETA.INV`; FI `F.INV`; FR `F.INV.RT`; LV `FINV` and
`BETAINV`.

`docs/research/t5107-fin-val4/search-fresh-predictions.tsv` (copied here) holds, for each cell, the engine's value
before the change and with it. They are hypotheses, never expected values.

## What it decides

- **Sheet D, 200 cells:** on each, Excel can equal the value with the change, the value before, or neither. The
  change is supported if it wins clearly, as it does on the cells already filled (hold-out census: the inverses go
  from 67.0% to 82.2% bit-exact on cells never read, the cdf's callers from 62.1% to 71.3%).
- **Sheet W, 320 cells:** the exact rate over the whole domain before and with the change, and whether any cell
  leaves 15 digits.
- If the change loses on a group, that group's piece does not land as written.
