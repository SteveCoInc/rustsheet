# Probe request: the stress suite's pending inputs (T-5313)

One case, 111 cells (column A: 82 formulas, column C: 29 typed values), each typed as one entry after load. Wanted: excel-graph values (`rustsheet-oracle fill -b excel-graph`). Map of cell -> stress case id: `cells.tsv`. Some inputs reach past M2 (DATEDIF, NETWORKDAYS, XLOOKUP, LAMBDA, SEQUENCE over 1e6 cells, 32,768-character REPT); the milestone tag is M2 per the excel-probe skill's rule for a mixed case, and the later-subject cells can be split out on landing.
