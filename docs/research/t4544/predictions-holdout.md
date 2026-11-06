# T-4544: predictions before the oracle fill (t4544-xmatch-error-modes-holdout)

Case: `eval/suites/probes/cases/t4544-xmatch-error-modes-holdout.case.json`. The engine's values (release `rustsheet-eval` on main fdc749398,
a scratch copy of the case with placeholder expectations), written before the fill. Each formula sits in A<r>;
the value column lists A<r>; A<r+1>; A<r+2> (a spill shows as two values, `<empty>` a blank).

| row | formula | A r; r+1; r+2 |
|---|---|---|
| 1 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{FALSE;-1},"z")` | #DIV/0!; #DIV/0!; <empty> |
| 4 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{1;9},2)` | #DIV/0!; #DIV/0!; <empty> |
| 7 | `=XMATCH(NA(),Data!$A$1:$A$3,{2;3},4)` | #N/A; #N/A; <empty> |
| 10 | `=XMATCH(NA(),Data!$A$1:$A$3,{1;1},"-1")` | #N/A; #N/A; <empty> |
| 13 | `=XMATCH("x"+1,Data!$A$1:$A$3,{"0";"0"},"z")` | #VALUE!; #VALUE!; <empty> |
| 16 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{0.5;-1.5},2.5)` | #DIV/0!; #DIV/0!; <empty> |
| 19 | `=XMATCH("x"+1,Data!$A$1:$A$3,{2;"0"},TRUE)` | #VALUE!; #VALUE!; <empty> |
| 22 | `=XMATCH(1/0,Data!$A$1:$A$3,{-1.5;0.5},1)` | #DIV/0!; #DIV/0!; <empty> |
| 25 | `=XMATCH(1/0,Data!$A$1:$A$3,{"0";-1},2)` | #DIV/0!; #DIV/0!; <empty> |
| 28 | `=XMATCH("x"+1,Data!$A$1:$A$3,{9;0},-1)` | #VALUE!; #VALUE!; <empty> |
| 31 | `=XMATCH(NA(),Data!$A$1:$A$3,{9;"0"},2.5)` | #N/A; #N/A; <empty> |
| 34 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{1;-3},0)` | #DIV/0!; #DIV/0!; <empty> |
| 37 | `=XMATCH("x"+1,Data!$A$1:$A$3,{0;1},4)` | #VALUE!; #VALUE!; <empty> |
| 40 | `=XMATCH("x"+1,Data!$A$1:$A$3,{-1;3},2)` | #VALUE!; #VALUE!; <empty> |
| 43 | `=XMATCH(NA(),Data!$A$1:$A$3,{9;FALSE},"z")` | #N/A; #N/A; <empty> |
| 46 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{-3;-1.5},4)` | #DIV/0!; #DIV/0!; <empty> |
| 49 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{0;3},1)` | #DIV/0!; #DIV/0!; <empty> |
| 52 | `=XMATCH(1/0,Data!$A$1:$A$3,{3;"y"},-2)` | #DIV/0!; #DIV/0!; <empty> |
| 55 | `=XMATCH(1/0,Data!$A$1:$A$3,{1;"y"},0)` | #DIV/0!; #DIV/0!; <empty> |
| 58 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,{-1;3},-2)` | #DIV/0!; #DIV/0!; <empty> |
| 61 | `=XMATCH("x"+1,Data!$A$1:$A$3,-1.5,{1;"-1"})` | #VALUE!; #VALUE!; <empty> |
| 64 | `=XMATCH("x"+1,Data!$A$1:$A$3,FALSE,{-2;1})` | #VALUE!; #VALUE!; <empty> |
| 67 | `=XMATCH("x"+1,Data!$A$1:$A$3,9,{"-1";0})` | #VALUE!; #VALUE!; <empty> |
| 70 | `=XMATCH(Data!$D$2,Data!$A$1:$A$3,0,{-2;0})` | #DIV/0!; #DIV/0!; <empty> |
| 73 | `=XMATCH(NA(),Data!$A$1:$A$3,9,{"z";-2})` | #N/A; #N/A; <empty> |
| 76 | `=XMATCH("x"+1,Data!$A$1:$A$3,1,{4;2.5})` | #VALUE!; #VALUE!; <empty> |
| 79 | `=XMATCH(1/0,Data!$A$1:$A$3,1,{-1;1})` | #DIV/0!; #DIV/0!; <empty> |
| 82 | `=XMATCH("x"+1,Data!$A$1:$A$3,3,{TRUE;TRUE})` | #VALUE!; #VALUE!; <empty> |
| 85 | `=XMATCH(1/0,Data!$A$1:$A$3,-1,{-2;1})` | #DIV/0!; #DIV/0!; <empty> |
| 88 | `=XMATCH(1/0,Data!$A$1:$A$3,0.5,{"-1";4})` | #DIV/0!; #DIV/0!; <empty> |
| 91 | `=XMATCH("x"+1,Data!$A$1:$A$3,"0",{TRUE;-1})` | #VALUE!; #VALUE!; <empty> |
| 94 | `=XMATCH("x"+1,Data!$A$1:$A$3,-1.5,{-1;2})` | #VALUE!; #VALUE!; <empty> |
| 97 | `=XMATCH(2,Data!$A$1:$A$3,2,#NULL!)` | #NULL!; <empty>; <empty> |
| 100 | `=XMATCH(2,Data!$A$1:$A$3,FALSE,#NULL!)` | #NULL!; <empty>; <empty> |
| 103 | `=XMATCH(2,Data!$A$1:$A$3,"0",#NAME?)` | #NAME?; <empty>; <empty> |
| 106 | `=XMATCH(2,Data!$A$1:$A$3,0.5,#DIV/0!)` | #DIV/0!; <empty>; <empty> |
| 109 | `=XMATCH(2,Data!$A$1:$A$3,3,#N/A)` | #N/A; <empty>; <empty> |
| 112 | `=XMATCH(2,Data!$A$1:$A$3,9,#NULL!)` | #NULL!; <empty>; <empty> |
| 115 | `=XMATCH(2,Data!$A$1:$A$3,9,#NUM!)` | #NUM!; <empty>; <empty> |
| 118 | `=XMATCH(2,Data!$A$1:$A$3,-3,#NAME?)` | #NAME?; <empty>; <empty> |
| 121 | `=XMATCH(2,Data!$A$1:$A$3,#NULL!,#REF!)` | #NULL!; <empty>; <empty> |
| 124 | `=XMATCH(2,Data!$A$1:$A$3,#REF!,#DIV/0!)` | #REF!; <empty>; <empty> |
| 127 | `=XMATCH(2,Data!$A$1:$A$3,#DIV/0!,#NULL!)` | #DIV/0!; <empty>; <empty> |
| 130 | `=XMATCH(2,Data!$A$1:$A$3,#N/A,#VALUE!)` | #N/A; <empty>; <empty> |
| 133 | `=XMATCH(2,Data!$A$1:$A$3,#NAME?,#VALUE!)` | #NAME?; <empty>; <empty> |
| 136 | `=XMATCH(2,Data!$A$1:$A$3,1,#NULL!)` | #NULL!; <empty>; <empty> |
| 139 | `=XMATCH(2,Data!$A$1:$A$3,0,#DIV/0!)` | #DIV/0!; <empty>; <empty> |
| 142 | `=XMATCH(2,Data!$A$1:$A$3,2,#NUM!)` | #NUM!; <empty>; <empty> |
| 145 | `=XMATCH(2,Data!$A$1:$A$3,FALSE,#REF!)` | #REF!; <empty>; <empty> |
| 148 | `=XMATCH(2,Data!$A$1:$A$3,#NAME?,#VALUE!)` | #NAME?; <empty>; <empty> |
| 151 | `=XMATCH(2,Data!$A$1:$A$3,0,#VALUE!)` | #VALUE!; <empty>; <empty> |
| 154 | `=XMATCH(2,Data!$A$1:$A$3,#DIV/0!,#N/A)` | #DIV/0!; <empty>; <empty> |
