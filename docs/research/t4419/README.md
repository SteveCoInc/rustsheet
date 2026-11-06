# T-4419: `.000` ties in [2^-19, 2^-18) days

Run the scripts from this directory with plain `python3`. They reuse T-4389's `common.py`, and through it T-4370's `model.py`.

- `gen_band.py` generated probes/t4419-band-ulp-map: 913 cells, 83 ties k = 0.1655 to 0.3295 s in steps of 2 ms, each
  stepped n = -19 to -9 ulp of k/86400. No model selected the cells. `meta1.json` holds, for each cell, the format,
  the scale, k, n, the serial's hex and the engine's prediction. The predictions were committed before the fill.
- probes/t4419-band-serials holds the same formulas without TEXT. Every serial Excel returns equals the generator's
  double.
- `search_pipelines.py` scores operation orders to milliseconds. The best is 887 of 913: (24,60,60,1000) with the half step in hours.
- `search_double15.py` scores 15-digit rounding done in doubles. The best is 869 of 913.

| model | band map (913) |
|---|---|
| engine (2^-72 days after the half step) | 847 |
| no offset (T-4370) | 838 |
| best operation order | 887 |
| best double 15-digit rounding | 869 |

The scoring is bit-exact text. No candidate is shipped, so no hold-out applies yet (decision 0188 R4). The task is
blocked as time-boxed (decision 0056), and its file records the next hypothesis.
