# Probe request `transition-evaluation`: what a sheet with "Transition formula evaluation" calculates

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For Taskei P559132976:
the engine ignores `sheetPr transitionEvaluation="1"`, and its owner wrote that the rules need probing in Excel
first. The research note beside this directory (`../transition-formula-evaluation-2026-11-11.md`) has what
Microsoft documents (S1: "Excel evaluates text strings as 0 (zero), Boolean expressions as 0 or 1, and database
criteria according to the rules that are used in Lotus 1-2-3") and what that sentence leaves open.*

## Why a file

The Office.js type definitions expose neither transition option (research note, S1; VBA has setters, on the
desktop), and the case builder writes no `sheetPr` attribute. The oracle tool takes a workbook file, so for
excel-graph the probe carries the setting in one generated workbook.

## How

- `python3 -I generate.py <dir>` writes `transition-evaluation.xlsx` (sha256
  `818dc595d7d972ec642e23f7b8468bb37e44b8d32256a90cafddbf1983d0fd6b`). It holds two sheets with the same cells:
  `L`, with `transitionEvaluation="1"`, and `E`, the control, without. Each row of the table below is one
  formula, in column Q of both sheets from row 2, with its id beside it in column P. The formula cells hold no
  cached value. It also writes `transition-evaluation-control.xlsx` (sha256
  `2a8af79162b54089512f3fc6ae4df81b7744b957d63a1578f3b372910029a324`): the same package with the attribute on
  neither sheet.
- With `<dir>` an absolute path (the command runs in `tools/`), the control first: `cd tools && uv run
  rustsheet-oracle recalc -b excel-graph <dir>/transition-evaluation-control.xlsx >
  <dir>/transition-evaluation-control.graph.json 2> <dir>/transition-evaluation-control.graph.err`, then the
  same with `transition-evaluation` for `transition-evaluation-control` in all three places. The tool's canary
  writer accepts both files (checked here, without the network). Please return both JSON files, and the `.err`
  file of a workbook the backend refused; the JSON is committed beside this README with a `RESULTS.md`.
- Nothing has opened a workbook of this generator in Excel yet, so a refusal is first told by the control. If
  the control is refused, the package is at fault and the attribute has not been asked. If the control
  calculates and the workbook with the attribute is refused, the refusal is about the attribute, and it is an
  answer.
- Each row answers by its two values: what `L!Q` holds against what `E!Q` holds. Where they are equal the option
  changes nothing for that formula.
- **If `L` and `E` agree on every row,** Excel for the web does not honour the attribute, and the result says
  only that. There is then nothing more to ask it, and no desktop Excel fills probes (the project lead's ruling
  of 2026-11-12; decision 0266 rule 2). What is left is the cached values of the files desktop Excel saved
  (docs/excel-semantics/xls-import.md, "Transition formula evaluation"), against a web that ignores the setting:
  the semantics README's `CONFLICTING` case, not a rule without an oracle, in which a task matches the web
  (decision 0039 rule 4). Whether to honour the setting against the web is then a question for the project
  lead (decision 0463).
- `python3 -I check.py` runs the generator and reads the workbook back: the two sheets hold the same inputs and
  each row's formula where the table says, only `L` has the attribute, the control differs by that attribute
  alone, the two names are defined, and the sha256s above are the files'. It needs no Excel and says nothing
  about Excel's values.

## What the engine gives today

`baseline-rustsheet.tsv` holds, for each row, the kind and the value the engine computes, measured on main
621a8367b by loading the generated workbook through the xlsx importer and calculating it
(`rustsheet::Workbook::from_xlsx_bytes` with `LoadOptions { recalc: true, .. }`, then the value of each `Q`
cell on both sheets; the review of CR-322048319 measured it again that way and got the same rows). The engine
ignores the option, so the two sheets agree on all 135 rows and the file has one value a row (an empty text is
written `""`). It is a record of that measurement, not a generated table, and `check.py` holds only that its
ids are the table's: against the oracle's answer for `E` it shows any row where the engine and Excel already
differ under ordinary rules, and against `L` what the option would change.

Row T20's value depends on the year of the run: `3/4` read as a date is the fourth of March of the current year.

## The cells both sheets hold

| Cells | Content |
|---|---|
| A1:A9 | `abc`; the text `12`; the empty string; 5; TRUE; (empty); `ABC`; `#N/A`; `abd` |
| B1:B7 | one space; the text `3/4`; the text `1E3`; the text `TRUE`; the text `0`; the text `5`; the formula `=""` |
| D1:E5 | a list: `Name`, `Qty`; `Apple` 1; `Apricot` 2; `apple` 4; `Ap` 8 |
| G1:L2 | six criteria ranges for it, each a field name over a criterion: `Name` `Ap`; `Name` `Apple`; `Name` `apple`; `Qty` `>1`; `Name` and the text `=Ap`; `Name` `A*` |
| N1:O3 | a lookup table sorted by its first column: the text `12` 1; `apple` 2; `Banana` 3 |
| S1:T2 | a lookup table of numbers: 5 `five`; 7 `seven` |
| U1:V7 | a list whose first column mixes kinds: `Item`, `Val`; `a` 1; `ab` 2; the number 5, 4; the number 10, 8; the empty string, 16; an empty cell, 32 |
| W1:AA2 | five criteria ranges for it: `Item` `a*`; `Item` and the text `=a`; `Item` `<>`; `Item` `>5`; `Crit` over the formula `=U2="a"` |

Two names are defined for the workbook: `OnL` is `L!$A$1+1` and `OnE` is `E!$A$1+1`.

## The rows

`OTHER` stands for the other sheet: `E` in the formulas of `L`, and `L` in those of `E`. The last column says
what the row asks; where it names a value "on both", that is what Excel's ordinary rules give, by our
understanding, and the row is a control.

| Id | Formula | What it asks |
|---|---|---|
| T1 | `=A1+1` | a text cell in addition: `#VALUE!`, or 1 |
| T2 | `=A1*2` | in multiplication |
| T3 | `=-A1` | under a unary minus |
| T4 | `=A1^2` | in a power |
| T5 | `=A1/2` | in a division |
| T6 | `=1-A1` | as the second operand |
| T7 | `="abc"+1` | a text literal, not a cell |
| T8 | `=A2+1` | a text cell that looks like a number: 13 as in Excel, or 1 |
| T9 | `="12"+1` | the same as a literal |
| T10 | `=A3+1` | a cell that holds the empty string |
| T11 | `=""+1` | the empty string as a literal |
| T12 | `=A1%` | under the percent operator |
| T13 | `=A1&1` | concatenation, a control: `abc1` on both |
| T14 | `=A6+1` | an empty cell, a control: 1 on both |
| T15 | `=A8+1` | an error cell, a control: `#N/A` on both |
| T16 | `=A1+A1` | text on both sides |
| T17 | `=+A1` | a unary plus: the text, or 0 |
| T18 | `=A5+1` | a Boolean cell in addition, a control: 2 on both |
| T19 | `=B1+1` | a cell that holds one space |
| T20 | `=B2+1` | the text `3/4`, which typed into a cell is a date |
| T21 | `=B3+1` | the text `1E3` |
| T22 | `=B4+1` | the text `TRUE` |
| T23 | `=B5+1` | the text `0` |
| T24 | `=B7+1` | a cell whose formula is `=""` |
| T25 | `=1&2` | concatenation of two numbers, a control: the text `12` on both |
| T26 | `=TRUE&""` | a Boolean made text: `TRUE`, or `1` |
| N1 | `=ROUNDUP(A1,0)` | text as a function's numeric argument (the corpus shows ROUNDUP roots disagreeing) |
| N2 | `=FIXED(A1,1)` | the same (FIXED is in the corpus list) |
| N3 | `=ABS(A1)` | the same, a plain math function |
| N4 | `=SUM(A1,1)` | text given to SUM as a reference argument: ignored on both, or 0 |
| N5 | `=SUM("abc",1)` | text given to SUM as a literal: `#VALUE!`, or 1 |
| N6 | `=AVERAGE(A1:A4)` | text cells in an averaged range: ignored (5), or counted as 0 (1.25) |
| N7 | `=COUNT(A1:A4)` | text cells in a counted range: 1, or 4 |
| N8 | `=COUNTA(A1:A4)` | a control |
| N9 | `=MAX(A1,-1)` | text as a reference argument of MAX |
| N10 | `=MIN(A1:A4)` | text cells in a range under MIN: 5, or 0 |
| N11 | `=SQRT(A1)` | text under a function with a domain |
| N12 | `=INT(A2)` | text that looks like a number as a numeric argument: 12, or 0 |
| N13 | `=VALUE(A1)` | VALUE of text that is no number: `#VALUE!`, or 0 |
| N14 | `=N(A1)` | N of text, a control: 0 on both |
| N15 | `=SUMPRODUCT(A1:A4,A1:A4)` | text cells in SUMPRODUCT, which already counts them as 0 |
| N16 | `=MOD(A1,3)` | text as the first argument of MOD |
| N17 | `=LEN(A1)` | a text function, a control: 3 on both |
| N18 | `=FIXED("1.5")` | text that is a number, as a literal argument of FIXED |
| N19 | `=ROUNDUP("2.1",0)` | the same for ROUNDUP |
| C1 | `=1<2` | the value of a comparison: TRUE, or 1 |
| C2 | `=TYPE(1<2)` | its type: 4 (logical), or 1 (number) |
| C3 | `=ISLOGICAL(1<2)` | the same question through ISLOGICAL |
| C4 | `=ISNUMBER(1<2)` | and through ISNUMBER |
| C5 | `=(1<2)+1` | a comparison in arithmetic, a control: 2 on both |
| C6 | `=AND(1<2,2<3)` | the value of AND |
| C7 | `=TYPE(AND(TRUE,TRUE))` | its type |
| C8 | `=OR(FALSE,1>2)` | the value of OR |
| C9 | `=NOT(1<2)` | the value of NOT |
| C10 | `=TRUE` | the constant TRUE as a formula's result |
| C11 | `=TYPE(TRUE)` | its type |
| C12 | `=ISNUMBER(5)` | the value of an IS function |
| C13 | `=TYPE(ISNUMBER(5))` | its type |
| C14 | `=A1="ABC"` | text compared with text in another case |
| C15 | `=A1>5` | text compared with a number: TRUE in Excel (text sorts after numbers), or 0>5 |
| C16 | `=A1=0` | is text equal to 0 |
| C17 | `=A2=12` | is text that looks like a number equal to that number |
| C18 | `=A1<A9` | two text cells compared |
| C19 | `=EXACT(A1,A7)` | a text function that returns a Boolean |
| C20 | `=TYPE(A5)` | the type of a Boolean constant in a cell: 4, or 1 |
| C21 | `=A5=1` | is a Boolean cell equal to 1 |
| C22 | `=ISLOGICAL(A5)` | is a Boolean cell logical |
| C23 | `=IF(1<2,"y","n")` | a comparison as a condition, a control: `y` on both |
| C24 | `=ISTEXT(A1)` | ISTEXT of a text cell |
| C25 | `=TYPE(A1)` | the type of a text cell: 2, or 1 |
| C26 | `=A1<1` | is text less than 1 |
| C27 | `="abc"=0` | a text literal compared with 0 |
| C28 | `=B5=0` | the text `0` compared with 0 |
| C29 | `=B5<1` | the text `0` compared with 1 |
| C30 | `=A6=0` | an empty cell compared with 0, a control: TRUE on both, unless its type changes |
| C31 | `=A6<1` | an empty cell compared with 1 |
| C32 | `=TRUE+TRUE` | Booleans added: 2, and its type |
| C33 | `=(1=1)*5` | a comparison multiplied |
| C34 | `=-TRUE` | a Boolean under a unary minus |
| C35 | `=SUM(TRUE,1)` | a Boolean literal given to SUM: 2 |
| C36 | `=SUM(A5,1)` | a Boolean cell given to SUM: ignored (1), or counted |
| I1 | `=IF(A1,1,2)` | a text cell as a condition: `#VALUE!`, or 2 (the corpus shows IF roots disagreeing) |
| I2 | `=IF(A6,1,2)` | an empty cell as a condition, a control: 2 on both |
| I3 | `=IF(A2,1,2)` | text that looks like a number as a condition |
| I4 | `=IF(A3,1,2)` | the empty string as a condition |
| I5 | `=ISBLANK(A1)` | ISBLANK of a text cell (ISBLANK is in the corpus list) |
| I6 | `=ISBLANK(A6)` | of an empty cell |
| I7 | `=ISBLANK(A3)` | of a cell that holds the empty string |
| I8 | `=TYPE(ISBLANK(A6))` | the type of its result |
| I9 | `=IF(A1+1,1,2)` | a condition computed from text |
| I10 | `=IFERROR(A1+1,"err")` | is there still an error to catch |
| I11 | `=ISERROR(A1+1)` | the same through ISERROR |
| I12 | `=IF(B5,1,2)` | the text `0` as a condition |
| I13 | `=A6` | a reference to an empty cell: 0, and its type |
| I14 | `=A6&"x"` | an empty cell made text: `x`, or `0x` |
| I15 | `=ISBLANK(B7)` | ISBLANK of a cell whose formula is `=""` |
| I16 | `=B7` | a reference to that cell: the empty string, or 0 |
| K1 | `=VLOOKUP("APPLE",N1:O3,2,FALSE)` | an exact lookup of text in another case (VLOOKUP is in the corpus list) |
| K2 | `=VLOOKUP(12,N1:O3,2,FALSE)` | a number looked up among text that looks like it |
| K3 | `=MATCH("banana",N1:N3,0)` | MATCH of text in another case |
| K4 | `=VLOOKUP("app",N1:O3,2,FALSE)` | the beginning of a key: `#N/A`, or a match |
| K5 | `=VLOOKUP("b",N1:O3,2,TRUE)` | an approximate lookup of text |
| K6 | `=MATCH(0,N1:N3,0)` | 0 looked up among text: `#N/A`, or the first text |
| K7 | `=VLOOKUP(A1,N1:O3,2,FALSE)` | a text cell as a key that is not in the table |
| K8 | `=INDEX(O1:O3,A5+1)` | a Boolean in an index, a control: 2 on both |
| K9 | `=VLOOKUP(B6,S1:T2,2,FALSE)` | a text key `5` looked up exactly in a table that holds the number 5 |
| K10 | `=VLOOKUP("5",S1:T2,2,FALSE)` | the same key as a literal |
| D1 | `=DSUM(D1:E5,"Qty",G1:G2)` | the criterion `Ap`: every name that begins with it (15), or the whole cell (8) |
| D2 | `=DSUM(D1:E5,"Qty",H1:H2)` | the criterion `Apple`: 5 in Excel (two cases of the word), or one case only |
| D3 | `=DCOUNT(D1:E5,"Qty",I1:I2)` | the criterion `apple` in lower case |
| D4 | `=DSUM(D1:E5,"Qty",J1:J2)` | a comparison criterion `>1`, a control: 14 on both |
| D5 | `=DSUM(D1:E5,"Qty",K1:K2)` | the criterion `=Ap`, which is exact in Excel: 8 |
| D6 | `=DSUM(D1:E5,"Qty",L1:L2)` | a wildcard criterion `A*` |
| D7 | `=DGET(D1:E5,"Qty",K1:K2)` | DGET with the exact criterion |
| D8 | `=DCOUNTA(D1:E5,"Name",G1:G2)` | DCOUNTA with the prefix criterion |
| D9 | `=DAVERAGE(D1:E5,"Qty",H1:H2)` | DAVERAGE with `Apple` |
| D10 | `=DSUM(D1:E5,2,G1:G2)` | the field given by its number |
| D11 | `=COUNTIF(D2:D5,"Ap")` | COUNTIF, which is exact for plain text: 1. Is it a "database criterion" too? |
| D12 | `=COUNTIF(D2:D5,"Ap*")` | COUNTIF with a wildcard: 4 |
| D13 | `=SUMIF(D2:D5,"Apple",E2:E5)` | SUMIF with text in two cases: 5 |
| D14 | `=COUNTIF(A1:A4,0)` | does a text cell count as 0 for a criterion |
| D15 | `=DSUM(U1:V7,"Val",W1:W2)` | the criterion `a*` over a column of text and numbers |
| D16 | `=DSUM(U1:V7,"Val",X1:X2)` | the criterion `=a` over it |
| D17 | `=DSUM(U1:V7,"Val",Y1:Y2)` | the criterion `<>` over it: which rows are not empty |
| D18 | `=DSUM(U1:V7,"Val",Z1:Z2)` | the criterion `>5` over it: does text count as 0, and is it then left out |
| D19 | `=DSUM(U1:V7,"Val",AA1:AA2)` | a criterion cell that holds a formula, `=U2="a"`, under a heading that is no field |
| D20 | `=COUNTIF(U2:U7,"a*")` | COUNTIF with the same four criteria, as controls or as further evidence |
| D21 | `=COUNTIF(U2:U7,"=a")` | COUNTIF with `=a` |
| D22 | `=COUNTIF(U2:U7,"<>")` | COUNTIF with `<>` |
| D23 | `=COUNTIF(U2:U7,">5")` | COUNTIF with `>5` |
| X1 | `=OTHER!A1+1` | a formula that reads a text cell of the other sheet: whose rules apply |
| X2 | `=SUM(OTHER!A1,1)+OTHER!A1` | the same with a function between |
| X3 | `=TYPE(OTHER!Q47)` | the type of the other sheet's comparison (its C1), read from here |
| X4 | `=OnL` | a name defined as `L!$A$1+1`, used from both sheets |
| X5 | `=OnE` | a name defined as `E!$A$1+1`, used from both sheets |

## What the answers decide

Each group of rows is one open point of the research note's S1. T and N say where a text string counts as 0; C
what a Boolean expression's value and type are; I what a condition and ISBLANK do; K whether lookups change; D
which criteria follow Lotus 1-2-3's rules and how; X whose rules apply when a formula and the cell it reads are
on sheets with different settings. The design (research note, S7) waits for them. Until then the engine's rule
is unchanged: every sheet is calculated by Excel's rules, and `docs/excel-semantics/xlsx-storage.md` records the
gap as UNVERIFIED.
