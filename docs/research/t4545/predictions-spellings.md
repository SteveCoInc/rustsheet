# T-4545: predictions before the oracle fill (t4545-indirect-sheet-prefix-spellings)

Case: `eval/suites/probes/cases/t4545-indirect-sheet-prefix-spellings.case.json`. The engine's values (release `rustsheet-eval` on main 98cfc5dad, a
scratch copy of the case with placeholder expectations), written before the fill. Columns B:E of sheet Summary read
the same INDIRECT text through ISREF, SUM, ERROR.TYPE and ROWS.

| row | INDIRECT argument | ISREF | SUM | ERROR.TYPE | ROWS |
|---|---|---|---|---|---|
| 1 | `"Données!B1"` | True | 7.0 | #N/A | 1.0 |
| 2 | `"'Données'!B1"` | True | 7.0 | #N/A | 1.0 |
| 3 | `"données!B1"` | True | 7.0 | #N/A | 1.0 |
| 4 | `"Ünïcode!B1"` | True | 7.0 | #N/A | 1.0 |
| 5 | `"'Ünïcode'!B1"` | True | 7.0 | #N/A | 1.0 |
| 6 | `"1Q!B1"` | True | 7.0 | #N/A | 1.0 |
| 7 | `"'1Q'!B1"` | True | 7.0 | #N/A | 1.0 |
| 8 | `"1q!B1"` | True | 7.0 | #N/A | 1.0 |
| 9 | `"Q1!B1"` | True | 7.0 | #N/A | 1.0 |
| 10 | `"x.y!B1"` | True | 7.0 | #N/A | 1.0 |
| 11 | `"2024!B1"` | True | 7.0 | #N/A | 1.0 |
| 12 | `"'2024'!B1"` | True | 7.0 | #N/A | 1.0 |
| 13 | `"2024!B1:2024!B3"` | False | #REF! | 4.0 | #REF! |
| 14 | `"Q1!B1:Q1!B3"` | False | #REF! | 4.0 | #REF! |
| 15 | `"'1Q'!B1:'1Q'!B3"` | False | #REF! | 4.0 | #REF! |
| 16 | `"1Q!B1:1Q!B3"` | False | #REF! | 4.0 | #REF! |
| 17 | `"Données!B1:Données!B3"` | False | #REF! | 4.0 | #REF! |
| 18 | `"Q1!B1:x.y!B3"` | False | #REF! | 4.0 | #REF! |
| 19 | `"2024!B1:B3"` | True | 24.0 | #SPILL! | 3.0 |
| 20 | `"'2024'!B1:'2024'!B3"` | False | #REF! | 4.0 | #REF! |
| 21 | `"Q1!B1:Q1!B3:Q1!B2"` | False | #REF! | 4.0 | #REF! |
| 22 | `"2024!R1C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 23 | `"'2024'!R1C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 24 | `"2024!R[-23]C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 25 | `"1Q!R1C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 26 | `"'1Q'!R1C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 27 | `"Données!R1C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 28 | `"Q1!R1C2",FALSE` | True | 7.0 | #N/A | 1.0 |
| 29 | `"2024!R1C2:R3C2",FALSE` | True | 24.0 | #SPILL! | 3.0 |
| 30 | `"2024!R1C2:2024!R3C2",FALSE` | False | #REF! | 4.0 | #REF! |
