# T-4543: predictions before the oracle fill

Case: `eval/suites/probes/cases/t4543-xlookup-not-found-address-sheet-lift.case.json`. The engine's values (release
`rustsheet-eval` on main 96340bc6, a scratch copy of the case with placeholder expectations), written before the fill.
Spill is the A:B grid of the block (rows r to r+2, `;` between rows, `<empty>` a blank); T is ROWS, U COLUMNS, V:Z
INDEX (1,1), (2,1), (1,2), (2,2), (3,1).

| row | formula | spill | T | U | V | W | X | Y | Z |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";"q"})` | "p", <empty>; "q", <empty>; <empty>, <empty> | 2.0 | 1.0 | "p" | "q" | #REF! | #REF! | #REF! |
| 5 | `=XLOOKUP(2,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";"q"})` | 20.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 20.0 | #REF! | #REF! | #REF! | #REF! |
| 9 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p","q"})` | "p", "q"; <empty>, <empty>; <empty>, <empty> | 1.0 | 2.0 | "p" | #REF! | "q" | #REF! | #REF! |
| 13 | `=XLOOKUP(2,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p","q"})` | 20.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 20.0 | #REF! | #REF! | #REF! | #REF! |
| 17 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2)` | "x", <empty>; "y", <empty>; <empty>, <empty> | 2.0 | 1.0 | "x" | "y" | #REF! | #REF! | #REF! |
| 21 | `=XLOOKUP(2,Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2)` | 20.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 20.0 | #REF! | #REF! | #REF! | #REF! |
| 25 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p","q";"r","s"})` | "p", "q"; "r", "s"; <empty>, <empty> | 2.0 | 2.0 | "p" | "r" | "q" | "s" | #REF! |
| 29 | `=XLOOKUP(2,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p","q";"r","s"})` | 20.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 20.0 | #REF! | #REF! | #REF! | #REF! |
| 33 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";"q"})` | 20.0, <empty>; "p", <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | "p" | #REF! | #REF! | #REF! |
| 37 | `=XLOOKUP({2;9;7},Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";"q"})` | 20.0, <empty>; "p", <empty>; "p", <empty> | 3.0 | 1.0 | 20.0 | "p" | #REF! | #REF! | "p" |
| 41 | `=XLOOKUP({9;2},Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";"q"})` | "p", <empty>; 20.0, <empty>; <empty>, <empty> | 2.0 | 1.0 | "p" | 20.0 | #REF! | #REF! | #REF! |
| 45 | `=XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2)` | 20.0, <empty>; #VALUE!, <empty>; <empty>, <empty> | 2.0 | 1.0 | 20.0 | #VALUE! | #REF! | #REF! | #REF! |
| 49 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";#N/A})` | "p", <empty>; #N/A, <empty>; <empty>, <empty> | 2.0 | 1.0 | "p" | #N/A | #REF! | #REF! | #REF! |
| 53 | `=XLOOKUP(2,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p";#N/A})` | 20.0, <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | 20.0 | #REF! | #REF! | #REF! | #REF! |
| 57 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$C$3,{"p";"q"})` | "p", <empty>; "q", <empty>; <empty>, <empty> | 2.0 | 1.0 | "p" | "q" | #REF! | #REF! | #REF! |
| 61 | `=XLOOKUP(2,Data!$A$1:$A$3,Data!$B$1:$C$3,{"p";"q"})` | 20.0, 200.0; <empty>, <empty>; <empty>, <empty> | 1.0 | 2.0 | 20.0 | #REF! | 200.0 | #REF! | #REF! |
| 65 | `=XLOOKUP(9,Data!$A$1:$A$3,Data!$B$1:$B$3,{"p"})` | "p", <empty>; <empty>, <empty>; <empty>, <empty> | 1.0 | 1.0 | "p" | #REF! | #REF! | #REF! | #REF! |
| 69 | `=ADDRESS(1,1,1,TRUE,{"S1";"S2"})` | "'S1'!$A$1", <empty>; "'S2'!$A$1", <empty>; <empty>, <empty> | 2.0 | 1.0 | "'S1'!$A$1" | "'S2'!$A$1" | #REF! | #REF! | #REF! |
| 73 | `=ADDRESS(1,1,1,TRUE,{"S1","S2"})` | "'S1'!$A$1", "'S2'!$A$1"; <empty>, <empty>; <empty>, <empty> | 1.0 | 2.0 | "'S1'!$A$1" | #REF! | "'S2'!$A$1" | #REF! | #REF! |
| 77 | `=ADDRESS(1,1,1,TRUE,Data!$D$1:$D$2)` | "x!$A$1", <empty>; "y!$A$1", <empty>; <empty>, <empty> | 2.0 | 1.0 | "x!$A$1" | "y!$A$1" | #REF! | #REF! | #REF! |
| 81 | `=ADDRESS({1;2},1,1,TRUE,{"S1";"S2"})` | "'S1'!$A$1", <empty>; "'S2'!$A$2", <empty>; <empty>, <empty> | 2.0 | 1.0 | "'S1'!$A$1" | "'S2'!$A$2" | #REF! | #REF! | #REF! |
| 85 | `=ADDRESS({1;2;3},1,1,TRUE,{"S1";"S2"})` | "'S1'!$A$1", <empty>; "'S2'!$A$2", <empty>; #N/A, <empty> | 3.0 | 1.0 | "'S1'!$A$1" | "'S2'!$A$2" | #REF! | #REF! | #N/A |
| 89 | `=ADDRESS({1;2},1,1,TRUE,Data!$D$1:$D$2)` | "x!$A$1", <empty>; "y!$A$2", <empty>; <empty>, <empty> | 2.0 | 1.0 | "x!$A$1" | "y!$A$2" | #REF! | #REF! | #REF! |
| 93 | `=ADDRESS({1;2;3},1,1,TRUE,Data!$D$1:$D$2)` | "x!$A$1", <empty>; "y!$A$2", <empty>; #N/A, <empty> | 3.0 | 1.0 | "x!$A$1" | "y!$A$2" | #REF! | #REF! | #N/A |
| 97 | `=ADDRESS(1,1,1,TRUE,{"S1";#N/A})` | "'S1'!$A$1", <empty>; #N/A, <empty>; <empty>, <empty> | 2.0 | 1.0 | "'S1'!$A$1" | #N/A | #REF! | #REF! | #REF! |
| 101 | `=ADDRESS(1,1,1,TRUE,{"S1";5})` | "'S1'!$A$1", <empty>; "'5'!$A$1", <empty>; <empty>, <empty> | 2.0 | 1.0 | "'S1'!$A$1" | "'5'!$A$1" | #REF! | #REF! | #REF! |
| 105 | `=ADDRESS(1,1,1,TRUE,{"S1";""})` | "'S1'!$A$1", <empty>; "!$A$1", <empty>; <empty>, <empty> | 2.0 | 1.0 | "'S1'!$A$1" | "!$A$1" | #REF! | #REF! | #REF! |
| 109 | `=ADDRESS(1,1,1,FALSE,{"S1";"S2"})` | "'S1'!R1C1", <empty>; "'S2'!R1C1", <empty>; <empty>, <empty> | 2.0 | 1.0 | "'S1'!R1C1" | "'S2'!R1C1" | #REF! | #REF! | #REF! |
