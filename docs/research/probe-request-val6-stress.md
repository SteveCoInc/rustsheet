# Probe request: the stress suite's pending inputs (T-5313)

One case, 111 cells (column A: 82 formulas, column C: 29 typed values), each typed as one entry after load. Wanted: excel-graph values (`rustsheet-oracle fill -b excel-graph`). Map of cell -> stress case id: `cells.tsv`. Some inputs reach past M2 (DATEDIF, NETWORKDAYS, XLOOKUP, LAMBDA, SEQUENCE over 1e6 cells, 32,768-character REPT); the milestone tag is M2 per the excel-probe skill's rule for a mixed case, and the later-subject cells can be split out on landing.

## Added later the same day
- `t5313-stress-session-inputs-addendum.case.json`: one typed entry, `#N/A`, typed into a cell.
- `t5313-3d-span-cell-like-end.case.json`: `=SUM(Sales:T3!C2)` (a 3-D span ending at a sheet named like a cell) and its quoted control, typed after load.
