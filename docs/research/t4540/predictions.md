# T-4540: ours before the Excel fill

probes/t4540-averageif-criteria-average-range, recorded from `rustsheet-eval show` on main 0db7e8ac9 before
the excel-graph fill (2026-11-06). A1:A6 = 0, -1, 0.5, text "1", TRUE, 1; D1:D6 = 10 to 60; E1 = 70. Columns H:J
take the criteria as a literal, K:M from G1:G7; H and K average D1:D6, I and L D1, J and M D1:E1 (both resized from
their top-left cell, criteria.md "SUMIF resizes sum_range from its top-left cell").

| row | criteria | ours, all six columns | rows matched |
|---|---|---|---|
| 1 | 0 | 10 | A1 |
| 2 | -1 | 20 | A2 |
| 3 | 0.5 | 30 | A3 |
| 4 | "1" (text) | 50 | A4 (text "1") and A6 (1) |
| 5 | "0.5" (text) | 30 | A3 |
| 6 | TRUE | 50 | A5 only, not A6's 1 |
| 7 | FALSE | `#DIV/0!` | none |
