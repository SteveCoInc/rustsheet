# Probe request: fresh cells for BETA.INV at a shape of exactly one (decision 0188 R4.2; T-4994, poc-gateb-cc)

One case, 253 checked cells, excel-graph, raw values:
`eval/suites/probes/cases/t4994-shape-one-fresh.case.json`, copied here at its suite path. It is not on main yet.

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t4994-shape-one-fresh.case.json
```

What comes back: `eval/suites/probes/cases/t4994-shape-one-fresh.oracle.json`.

## Why

BETA.INV is one of the five functions that still fail the expert tier gate (decision 0355), and after the math
values of T-5491 and T-5498 it fails only its type bar: 39 cells of a wrong type, 36 allowed. Twenty-four of them
are one family, probes/t4974-beta-inv-tiny-shape-num-boundary: Excel returns a number (0.9999999999999999, 2^-53,
0.999999) and the engine `#NUM!`. I traced the engine's search on those inputs: it stops exactly on Excel's value.
The engine does not return it because at a shape of exactly one it takes a closed form first.

The rule that follows: at a shape of one the closed form is Excel's value only where it is a double strictly
inside (0, 1) in AS 109's orientation (the flip at p > 1/2); elsewhere Excel returns its search's own stop. On the
124 shape-one cells of three filled probes (t4974 tiny-shape, t4713, t3351) the engine goes from 36 bit-exact and
20 of a wrong type to 55 and 2. It is a four-line change on top of the BETA.INV search change (the one that waits
for val9), so decision 0188 R4.2 owes it Excel values drawn after it was frozen.

## What is in it

Every argument is a literal of at most 15 significant digits; each is the nearest double under the engine's model
of Excel's literal read (759 literals, none off). Strata, none chosen by Excel's answer:

| Sheet | Cells | What it is | The engine with the rule predicts |
|---|---:|---|---|
| OneTiny | 60 | alpha = 1, beta from 1E-20 to 0.3, p on both sides of 1/2 | 29 `#NUM!`, 10 at 1 - 2^-53, 4 at 0.999999, 17 other numbers |
| TinyOne | 60 | the shapes exchanged | 35 `#NUM!`, 12 at 2^-53, 3 near 1E-6, 10 other numbers |
| OneLarge | 40 | one shape 1, the other from 3 to 8E4 | 40 numbers (the closed form) |
| Edge | 78 | the other shape chosen so the oriented closed form is 1 - 2^-k, k from 40 to 1000 | 24 at 1 - 2^-53, 12 at 2^-53, 42 other numbers |
| Legacy | 15 | BETAINV on rows of the sheets above | the same values |

Frozen before the fill, in `docs/research/t4994/` here: `fresh-engine.tsv` (the engine's value of every cell, branch
`gateb/shape-one`) and the generator `fresh.py` (seed 4994_2026_11_11).

## Read of the fill, fixed here

The rule is confirmed if Excel has an error where the engine predicts one and elsewhere a number within 1E-12 of
the prediction, relative. The many `#NUM!` predictions of OneTiny and TinyOne are the search not stopping; a number
from Excel there is a miss of the search model, not of the shape-one rule, and is reported apart.

No urgency beyond the next batch.
