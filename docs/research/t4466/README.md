# T-4466 / T-4333: the elapsed count before a fraction of a second at a tie

Run the scripts from this directory with plain `python3`. Order of work (decision 0188 R4):

1. `gen.py` built probes/t4466-count-hypotheses (536 cells) and `predict.py` wrote `predictions.json`
   (the engine's register, T-4333's two hypotheses) before the fill; both committed before excel-graph filled it
   (2026-11-08). `score.py` scores them.
2. `count_models.py` scores count models on every elapsed-count cell with fraction digits of the t4323, t4332,
   t4333, t4370, t4389 and t4466 probes (2017 cells), with a 30% hold-out by cell hash. Its `max` model was frozen
   here, then `gen_fresh.py` drew probes/t4466-count-fresh (600 cells, random.Random(44661)) and wrote
   `fresh_predictions.json` before the fill. `score_fresh.py` scores it.

## What probes/t4466-count-hypotheses shows
Rows 1-40 step the eight t4333 ulp-scan seeds by -2..+2 ulp under `[s]`, `[s].0`, `[s].00`, `[s].000`, `[m]:ss.0`,
`[h]:mm:ss.0` and `[s].0 General` (columns A to G).
- The bare `[s]` rounds to the nearest second on every row (A1:A40) and never flips with the fraction forms. Beside
  `.00` and `.000` the `.95` serials are not ties, and the counts agree with the fraction (C, D).
- The count carries per unit, on its own: on `45000+59.95/86400` the `[s]` and `[m]` counts carry
  ("3888000060.9", "64800001:59.9", B18, E18) and the clock does not; on `60000+82799.95/86400` the `[h]` count
  carries ("1440023:59:59.9", F33) while `[s]` and `[m]` do not (B33, E33).
- The count is not a function of the half-stepped serial `y`. `3733.95/86400+1*2^-57` and `+2*2^-57` print the
  same register, "1.63425E-12" (G29, G30), yet their counts are "3733.0" and "3734.0" (B29, B30).
- The `General` register beside fraction digits is the field cut's remainder of a second times the scale (72 of 72
  cells of G1:G40 and B121:B152), not `frac(y)*86400*s` cut at once (53).
- Rows 41-120 move the seeds to their `.00` and `.000` ties: the same "count + 1 beside .99" (A42, A48, A53, A57)
  and "count kept beside a wrapped .00" (A68, `3733.995/86400` "3733.00") occur there.
- Hold-out rows 121-152: 32 fresh `d + k.95` ties at the nearest double. Excel's count carries in 25 of them.

Scores of the pre-fill predictions (`score.py`), count cells, train rows 1-120 / hold-out rows 121-152:

| model | train | hold-out |
|---|---|---|
| engine register `floor(y*86400/u)` | 374/400 | 72/96 |
| H1: the nearest double is the decimal tie, rounded half up | 380/400 | 76/96 |
| H2: bare `[s]` rounding with a carry test against the shown fraction | 378/400 | 64/96 |

Both hypotheses of T-4333 are refuted: neither gets every cell, and neither beats the register by enough on the
hold-out to adopt it.

## Count models (`count_models.py`, 2017 cells)
| model | train (1407) | hash hold-out (610) | all |
|---|---|---|---|
| correctly rounded: the exact value rounded half up to the shown digit | 1343 | 584 | 1927 |
| register `floor(y*86400/u)` (engine before this change) | 1352 | 598 | 1950 |
| chain60 `floor(((y*60)*1440)/u)` | 1384 | 603 | 1987 |
| max of the two (frozen before t4466-count-fresh; superseded) | 1376 | 605 | 1981 |

The hold-out is sha256("case!cell") mod 10 below 3, fixed before scoring; no margin was stated in advance, and
`max`'s hold-out rate (99.2%) is above its train rate (97.8%). The fresh sweep is kept out of this split.
`max` moves none of the register's 1950 cells; chain60 alone misses 30 cells, some of them cells the register gets. Searched first, without success: every
three-operation expression in `x` or `frac(x)` over the units, the scale and the half step (best 1987, chain60),
tick counts of 1E-3 to 1E-9 s and 2^-20 to 2^-30 s truncated or rounded (best 1960), decimal forms of x or
x*86400 at 15, 16 and 17 digits or shortest round trip (best 1954), reciprocal constants `1/86400` (best 1508 of the
first 1521), and the register with a half step off by up to 3E-6 s in double, exact or 80-bit arithmetic.

## Register fresh sweep (`score_register_fresh.py`, probes/t4466-register-fresh, drawn after the freeze)
`gen_register_fresh.py` (random.Random(44662)) drew 120 `TEXT(x,"<date section>.0|.00|.000 General")` cells after
the field-cut register was in the engine; the engine's strings were committed in `register_fresh_predictions.json`
before excel-graph filled it (2026-11-08). The engine matches Excel on 120 of 120 cells; the serial cut at once is
within 1E-9 of Excel's register on 119 (it misses A71, "31.0 1.01409E-09"). The sweep holds fewer ties than the
t4466 rows, so it separates the two registers on one cell only.

## Fresh sweep (`score_fresh.py`, probes/t4466-count-fresh, drawn after the freeze)
| model | right of 600 |
|---|---|
| correctly rounded | 415 |
| register | 545 |
| max | 557 |

The two differ on 20 cells; `max` is right on 16. Excel's count still differs from both on 43 cells, at every unit
and digit count, for example `TEXT(366+68399.95/86400,"[h]:mm:ss.0")` "8803:59:59.9" (A12) and
`TEXT(45001+11950.95/86400,"[s].0")` "3888098350.0" (A136). The operation behind the count is still unknown; T-4333
keeps those cells.

## The chain rule (adopted, T-4333)
`count_chain.py` searches every chain of products whose product is the unit's count per day, crossed with the forms of
the half step: `((y*60)*60)*24` for `[s]`, `(y*60)*24` for `[m]` and `y*24` for `[h]` score 2,617 of 2,617 count cells,
t4466-count-fresh included (which the search therefore used, so it is not the fresh sweep for this rule). Hold-out:
1,407 / 1,407 train and 610 / 610 on the split above. Fresh: `gen_chain_fresh.py` drew probes/t4466-chain-fresh with a
new seed after the rule was frozen and wrote chain_fresh_predictions.json before the fill: chain 600, register 556,
rounded 490 of 600. The "operation still unknown" note above is superseded.

## General's fixed-or-E choice
`gen_general_fresh.py` drew probes/t4466-general-fixed-or-e (values in [1E-09, 1E-04) where the trimmed-digit rule and
the room rule disagree, plus controls) and wrote general_predictions.json before the fill: room rule 78 / 80, trimmed
rule 2 / 80, controls 38 / 40 each. The 4 misses showed the two refinements the engine then took: fixed's room runs
from the value's leading digit (a carry gains none), and the E mantissa rounds the 15-digit decimal half away.

