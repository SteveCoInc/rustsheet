# T-4545: predictions before the oracle fill (t4545-indirect-sheet-prefix-names)

Case: `eval/suites/probes/cases/t4545-indirect-sheet-prefix-names.case.json`. The engine's values (release `rustsheet-eval` on main 98cfc5dad, a
scratch copy of the case with placeholder expectations), written before the fill. Columns B:E of sheet Summary read
the same INDIRECT text through ISREF, SUM, ERROR.TYPE and ROWS.

| row | INDIRECT argument | ISREF | SUM | ERROR.TYPE | ROWS |
|---|---|---|---|---|---|
| 1 | `"Données!tn"` | True | 8.0 | #N/A | 1.0 |
| 2 | `"'Données'!tn"` | True | 8.0 | #N/A | 1.0 |
| 3 | `"1Q!tn"` | True | 8.0 | #N/A | 1.0 |
| 4 | `"'1Q'!tn"` | True | 8.0 | #N/A | 1.0 |
| 5 | `"1q!tn"` | True | 8.0 | #N/A | 1.0 |
| 6 | `"Ünïcode!tn"` | True | 8.0 | #N/A | 1.0 |
| 7 | `"Q1!tn"` | True | 8.0 | #N/A | 1.0 |
| 8 | `"2024!tn"` | True | 8.0 | #N/A | 1.0 |
