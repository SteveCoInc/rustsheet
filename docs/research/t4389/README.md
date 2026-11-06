# T-4389: `.000` fraction digits take a 2^-72-day offset on the serial

Run these scripts from this directory with plain `python3`. They reuse T-4370's `model.py` (and through it T-4332's
`chain.py` and T-4323's `frozen_fmt.py`).

- `common.py` holds T-4370's digits and the two candidates: 2^-72 days (1.83E-17 s) added to the remainder in
  seconds, or added to the serial in days after the half step. Both were written down before the hold-out existed.
  The constant is FITTED: every offset in [1.74E-17, 1.86E-17] s scores the 538 `.000` cells of the t4323, t4332 and
  t4370 fraction cases (T-4370's model 476), and 2^-72 days is the power of two inside that window.
- `gen_holdout.py` generated probes/t4389-holdout (368 cells, seed 43890; meta10.json). It has three blocks:
  - 40 `.000` ties below 1 s, each stepped down to 4.5E-17 s below the tie;
  - 31 ties after whole seconds or minutes where the two candidates disagree;
  - `.00`/`.0` controls.
  The margin was fixed before scoring: within 10 points of the train rate, and above T-4370 and the correctly rounded
  value.
- `gen_fresh.py` generated probes/t4389-fraction-fresh (120 cells, seed 43891; meta11.json) after the days variant
  was frozen. The first 60 cells are ones where the frozen model and T-4370 disagree.
- `gen_confirm.py` generated probes/t4389-confirm (80 cells, seed 43892; meta12.json) after the excel-fidelity review
  of #3440, with the ungated days model as the prediction fixed beforehand:
  - A1:A40: 40 ties whose half-stepped serial is in [2^-19, 2^-18) days. There 2^-72 is half an ulp, and the cells
    separate the ungated model from a gate at 2^-19 days (the first engine version).
  - A41:A80: 40 new ties after whole seconds where days and seconds disagree.
- `python3 score.py` scores train and hold-out. `python3 score.py ../t4389/meta11.json:t4389-fraction-fresh` (or
  `meta12.json:t4389-confirm`) scores a batch. "C in days (frozen)" is exactly the engine's `millisecond_offset`.

| cells | days (frozen) | seconds | T-4370 | correctly rounded |
|---|---|---|---|---|
| train, `.000` | 538/538 | 538/538 | 476/538 | 273/538 |
| train, all 1668 | 1664 | 1664 | 1602 | 1111 |
| hold-out | 368/368 | 337/368 | 337/368 | 81/368 |
| fresh sweep | 120/120 | 120/120 | 42/120 | 39/120 |
| confirm, band A1:A40 | 29/40 | 11/40 | 11/40 | 11/40 |
| confirm, whole seconds A41:A80 | 40/40 | 0/40 | 40/40 | 20/40 |

On the band cells, a gate at 2^-19 days scores 11/40, the same as T-4370. The 11 band misses
(A4, A6, A9, A14:A19, A31, A36) are T-4419's.

All counts are bit-exact text. The within-4-ulp and 15-digit columns of R4.1 do not apply to TEXT output. The
hold-out is a newly generated probe, not a 30% split of existing cells, because every existing `.000` cell was
used in the fit.

The four train misses are not this task's: three `[s]` counts (T-4333) and t4370-ulp-map-2 A418, the "42.0" wrap
cell. That cell rests on excel-graph alone and is decided within an ulp.

Gate A: no Gate A envelope applies. These are TEXT outputs, which are judged on the exact text and have no math
sidecar.

Not found: an operation that would add 2^-72 days under `.000` only. The `.000` half step is 0.5/86400000 days; its
15-digit decimal form differs from it by 3E-24 days, and float32 by about 2E-16, so neither produces 2.1E-22 days.
