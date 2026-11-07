# T-4793: GROWTH with several x, and a subnormal product of normal factors

`gen.py` builds probes/t4793-growth-several-x and `predictions.tsv`. Both were committed before the excel-graph fill.
H1 models the engine as T-4765 left it. The engine's actual values are checked against H1 after the fill.

- **Multi.** The four points are y = e^(1 + m1 x1 + m2 x2) at (1,1), (2,1), (1,2) and (2,3). known_y is a column
  against the 4x2 known_x, and the new x is (2,2). The slope terms are opposing and large, from ±200 to ±800.
  - H1, a single summed slopes' factor: e^1 every time.
  - H3, one factor per variable: `#NUM!` once one of them overflows.
  - A4 (±200) is a control, where H1 and H3 agree.
- **SubProd.** The points are y = e^(c + m x) at x = 0.5, 1 and 1.5, and the new x is 2. b = e^c and F = e^(2m) are
  both normal, while b·F is near or below DBL_MIN.
  - H1: e^(c + 2m) as it is, subnormal or not.
  - H4: the product is flushed to 0 once it is subnormal (numbers.md "Subnormal results become 0").
  - A1 and A5 are normal controls.

Run: `python3 -I docs/research/t4793/gen.py eval/suites/probes/cases/t4793-growth-several-x.case.json docs/research/t4793/predictions.tsv`
