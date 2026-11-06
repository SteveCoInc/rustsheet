# T-4563: predictions before the oracle fill

Case: `eval/suites/probes/cases/t4563-xlookup-not-found-ref-element.case.json`. These are the engine's values before any
T-4563 change. They were computed by the release `rustsheet-eval` built from T-4543's head 3f9f796, which has no engine
change over main, on a scratch copy of the case with placeholder expectations, and written before the fill.

Columns:
- **spill:** the block's A:B grid, rows r to r+2. `;` separates rows and `<empty>` is a blank.
- **T:** ROWS. **U:** COLUMNS.
- **V to Z:** INDEX at (1,1), (2,1), (1,2), (2,2) and (3,1).

| row | formula | spill | T | U | V | W | X | Y | Z |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `=ISREF(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2),2,1))` | FALSE, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | FALSE | #REF! | #REF! | #REF! | #REF! |
| 5 | `=ISREF(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),2,1))` | FALSE, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | FALSE | #REF! | #REF! | #REF! | #REF! |
| 9 | `=ISREF(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1),2,1))` | FALSE, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | FALSE | #REF! | #REF! | #REF! | #REF! |
| 13 | `=INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),2,1)` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 17 | `=INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1),2,1)` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 21 | `=INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Other!$A$1:$A$2),2,1)` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 25 | `=INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2),2)` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | #VALUE! | #REF! | #REF! | #REF! | #REF! |
| 29 | `=INDEX(XLOOKUP({9;2},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),1,1)` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 33 | `=SUM(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),2,1))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 37 | `=TYPE(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),2,1))` | 16.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 16.0 | #REF! | #REF! | #REF! | #REF! |
| 41 | `=TYPE(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1),2,1))` | 16.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 16.0 | #REF! | #REF! | #REF! | #REF! |
| 45 | `=AREAS(INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),2,1))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 49 | `=CELL("address",INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2),2,1))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 53 | `=CELL("address",INDEX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1),2,1))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 57 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2)+0` | 20.0, <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | #VALUE! | #REF! | #REF! | #REF! |
| 61 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1)+0` | 20.0, <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | #VALUE! | #REF! | #REF! | #REF! |
| 65 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2)&""` | "20", <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | "20" | #VALUE! | #REF! | #REF! | #REF! |
| 69 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2)*{1;1}` | 20.0, <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | #VALUE! | #REF! | #REF! | #REF! |
| 73 | `=SUM(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 77 | `=COUNT(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | 1.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 1.0 | #REF! | #REF! | #REF! | #REF! |
| 81 | `=COUNTA(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | 2.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 2.0 | #REF! | #REF! | #REF! | #REF! |
| 85 | `=ISERROR(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | FALSE, <empty>; TRUE, <empty>; <empty>, <empty> | 2.0 | 1.0 | FALSE | TRUE | #REF! | #REF! | #REF! |
| 89 | `=ISREF(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | FALSE, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | FALSE | #REF! | #REF! | #REF! | #REF! |
| 93 | `=ISTEXT(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2))` | FALSE, <empty>; FALSE, <empty>; <empty>, <empty> | 2.0 | 1.0 | FALSE | FALSE | #REF! | #REF! | #REF! |
| 97 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1)` | 20.0, <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | #VALUE! | #REF! | #REF! | #REF! |
| 101 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Other!$A$1:$A$2)` | 20.0, <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | #VALUE! | #REF! | #REF! | #REF! |
| 105 | `=TYPE(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | 64.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 64.0 | #REF! | #REF! | #REF! | #REF! |
| 109 | `=SUM(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$E$1))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
| 113 | `=MAX(XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$F$1:$F$2))` | #VALUE!, <empty>; <empty>, <empty>; <empty>, <empty> | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! | #VALUE! |
