# T-4538: predictions before the oracle fill

Case: `eval/suites/probes/cases/t4538-indirect-a1-cells.case.json` (H2 = 5). Column B asks `INDIRECT("H2",a1)`
(A1 text), column C asks `INDIRECT("R2C8",a1)` (R1C1 text), so a TRUE a1 reads 5 in B and `#REF!` in C, and a
FALSE a1 the other way round. Predictions are the engine's, read from `indirect_of` and `cast_bool_ref`
(crates/rustsheet-engine/src/cast.rs) on main 257793681, written before the fill.

| row | a1 | predicted a1 | B | C |
|---|---|---|---|---|
| 20 | `"FALSE"` | FALSE | #REF! | 5 |
| 21 | `"false"` | FALSE | #REF! | 5 |
| 22 | `"True"` | TRUE | 5 | #REF! |
| 23 | `" TRUE"` | #VALUE! | #VALUE! | #VALUE! |
| 24 | `"TRUE "` | #VALUE! | #VALUE! | #VALUE! |
| 25 | `"1"` | #VALUE! | #VALUE! | #VALUE! |
| 26 | D1 (TRUE) | TRUE | 5 | #REF! |
| 27 | D2 (FALSE) | FALSE | #REF! | 5 |
| 28 | D3 (0) | FALSE | #REF! | 5 |
| 29 | D4 (1) | TRUE | 5 | #REF! |
| 30 | D5 (-0.5) | TRUE | 5 | #REF! |
| 31 | D6 (0.5) | TRUE | 5 | #REF! |
| 32 | D7 (text "FALSE") | FALSE | #REF! | 5 |
| 33 | D8 (text "1") | #VALUE! | #VALUE! | #VALUE! |
| 34 | D9 (`=""`) | #VALUE! | #VALUE! | #VALUE! |
| 35 | D10 (blank) | FALSE | #REF! | 5 |
| 36 | D11 (`=NA()`) | #N/A | #N/A | #N/A |
| 37 | D12 (text "TRUE") | TRUE | 5 | #REF! |
| 38 | D13 (text "false") | FALSE | #REF! | 5 |
| 39 | D2:D2 | FALSE | #REF! | 5 |
| 40 | D1:D1 | TRUE | 5 | #REF! |
| 41 | D10:D10 | FALSE | #REF! | 5 |
| 42 | omitted (`INDIRECT("H2",)`) | FALSE | #REF! | 5 |
| 43 | ERROR.TYPE of row 36 | | 7 | 7 |

The open questions are rows 33-35 and 42: text "1" and empty text read from a cell (a cell's text may be read
differently from a literal), a blank cell, and an omitted a1, which in Excel may read as FALSE or as the default TRUE.

## Result (excel-graph, 2026-11-06)

Excel returned the predicted value in all 48 formula cells (B20:C43), so the engine already matches. The questions
that were open read: the text "1" from a cell and `=""` are `#VALUE!` (rows 33-34); a blank cell and an omitted a1
are FALSE (rows 35, 41, 42), so `INDIRECT("H2",)` reads "H2" as R1C1 and is `#REF!`.
