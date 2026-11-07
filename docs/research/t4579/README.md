# T-4579: paired T.TEST, probe predictions

`gen.py` (seed 457920261107) builds probes/t4579-ttest-paired and `predictions.tsv`, both committed before the
excel-graph fill. Every input is a 15-digit literal. The predictions use Python's nearest doubles. These equal Excel's x87 literal reads
except for one literal in Div!A21, which leaves its differences unchanged.

- **Div** (27 rows): 2 to 4 pairs with near-constant differences. Rows are classified by the one-pass variance
  `(Σd² - (Σd)²/n)/(n-1)` of the double differences:
  - 8 with that variance negative, 8 with it exactly zero, 8 with it tiny positive;
  - 3 controls whose double differences are exactly constant.

  H1: `#DIV/0!` exactly when the one-pass variance is <= 0 (A136's hypothesis). H2: exactly when the two-pass
  variance is <= 0 (the engine before T-4579). The two differ on the 16 neg/zero rows.
- **Off** (24 rows): x = a + u and y = b + u + shift + noise, with offsets a and b from 1E3 to 1E6, a shared spread u,
  and a paired effect from 1E-9 to 1E-6. The predicted two-tailed p comes from three candidates:
  - double differences (the engine before T-4579);
  - exact differences (any extended-precision subtraction that is exact here);
  - per-sample deviations `(x - m_x) - (y - m_y)`.

  They differ by more than 1E-12 relative on 21 of the 24 rows.
- **Rev**: the reversed argument orders the task names, recorded as found.

Nothing is tuned to the fill. Each hypothesis is kept or rejected by the cells.

Run (from the repository root; mpmath comes from the tools venv):
`cd tools && uv run python -I ../docs/research/t4579/gen.py ../eval/suites/probes/cases/t4579-ttest-paired.case.json ../docs/research/t4579/predictions.tsv`
