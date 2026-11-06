# T-4569 predictions (written before the oracle fill)

Case: `probes/t4569-xlookup-ref-element-max-min`. `X(r)` = `XLOOKUP({2;9},A1:A3,B1:B3,r)`: element 1 is the hit,
20; element 2 is the range `r`, kept as a reference (T-4563). G1:G2 = 50, 60 (above the hit). H1:H2 = 1, 2 (below).

The T-4563 evidence points two ways. SUM walks the reference element (`SUM(X(F1:F2))` 23 = 20 + 3 + 4), but COUNT
is 1 and COUNTA 2, so those count the element as one opaque item. The prediction follows SUM for the
reference-taking aggregates:

| cell | formula | predicted (walks, like SUM) | if it does not walk |
|---|---|---|---|
| A1 | `MAX(X(G1:G2))` | 60 | 20 |
| A2 | `MIN(X(H1:H2))` | 1 | 20 |
| A3 | `MAXA(X(G1:G2))` | 60 | 20 |
| A4 | `MINA(X(H1:H2))` | 1 | 20 |
| A5 | `MAX(X(G1))` | 50 | 20 |
| A6 | `MIN(X(H1))` | 1 | 20 |
| A7 | `AVERAGE(X(G1:G2))` | 43.333… (130/3) | 20 |
| A8 | `PRODUCT(X(H1:H2))` | 40 | 20 |
| A9 | `COUNTA(X(G1:G2))` | 2 (as T-4563) | 2 |
| A10 | `COUNT(X(G1:G2))` | 1 (as T-4563) | 1 |
| A11 | `SUMSQ(X(H1:H2))` | 405 | 400 |
| A12 | `LARGE(X(G1:G2),1)` | 20 (an array reader sees `#VALUE!` and skips it? uncertain; may be `#VALUE!`) | |
| A13 | `SMALL(X(H1:H2),1)` | 20 (same uncertainty) | |
| A14 | `MAX(X(G1:G2),0)` | 60 | 20 |
| A15 | `MIN(X(H1:H2),100)` | 1 | 20 |
| A16 | `MAX(X(G1:G2)+0)` | `#VALUE!` (a lifted operator reads the element as `#VALUE!`, T-4563) | |
| A17 | `MAX(INDEX(X(G1:G2),2,1))` | 60 (INDEX hands the reference back) | |

## Round 2 (rows 20-34), predicted before its fill, after round 1 showed top-left reading

Round 1's answer, 50 for `MAX(X(G1:G2))` and 401 for SUMSQ, also reconciles T-4563. Its `SUM(X(F1:F2))` = 23 is
20 + F1, the top-left cell, not 20 + 3 + 4 (that is 27). So SPEC-0014 rule 4 (top-left) holds for the reference
element of a mixed grid. Predictions:

- `SUM(X(G1:G2))` 70.
- `SUMPRODUCT` 20: rule 5 reads the reference element as 0.
- MEDIAN and STDEV.S: `#N/A` like LARGE (uncertain), or top-left (35, 21.2132…).
- Per-element callers (ROW, AREAS, COUNTIF, N, SUBTOTAL, COUNTBLANK, CELL) lift to 2x1. Element 2 is the call on
  G1:G2: ROW 1, AREAS 1, COUNTIF 2, N 50, SUBTOTAL 110, COUNTBLANK 0, CELL 1. Element 1 is the call on the value
  20: `#VALUE!` where a reference is required (ROW, AREAS, COUNTIF, SUBTOTAL, COUNTBLANK, CELL), N 20.
- ISREF FALSE, and ISNUMBER lifted {TRUE; FALSE}.
- `LARGE(OFFSET(G1,{0;1},0),1)` and SMALL: `#N/A`, as in round 1 (SPEC-0014 left these open).
