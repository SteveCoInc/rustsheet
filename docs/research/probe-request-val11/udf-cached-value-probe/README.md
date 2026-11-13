# UDF and XLM cells: does Excel for the web keep a file's cached value until a precedent changes?

From poc-engine-c-cc, 2026-11-12, for the coordinator's ruling of 12:02Z. That ruling says a cell whose formula needs
a function the engine cannot evaluate keeps its cached value on load, until a precedent changes. If the web
disagrees, we follow the web. Corpus rows #11 (VBA user-defined functions, 14 files) and #15 (`GET.WORKBOOK` in a
defined name, 10 files).

Kind: stateful (open, read, edit, read), like t5145-recalc-probe. Office.js or by hand in Excel for the web, en-US.

## Input

`udf-cached-value.xlsx` (SHA-256 `cb62d3c488a72729f4c4c25f934ce8662b6d7b65219c26309e1358ce1aea8b29`) is a plain
.xlsx built by hand. No VBA project defines `MyUdf`, as in a macro-enabled file whose code the web does not run.
The web can neither author a VBA project nor run one, so an .xlsm adds nothing the web could use. Sheet1, with the
cached value of each cell as stored:

| cell | formula | cached |
|---|---|---|
| A1 | 5 (a constant) | |
| B1 | `=MyUdf(A1)` | 42 |
| C1 | `=B1*2` | 84 |
| D1 | `=MyUdf(G1)` | 7 |
| E1 | `=INDEX(SheetList,1)`, with the name `SheetList` = `GET.WORKBOOK(1)` (XLM) | `[udf-cached-value.xlsx]Sheet1` (text) |
| F1 | `=ISERROR(B1)` | FALSE |
| G1 | 1 (a constant) | |
| H1 | 100 (a constant, nothing reads it) | |

## Steps

1. Upload the file and open it. Record B1, C1, D1, E1 and F1 as shown, and each one's formula bar.
2. Type 200 in H1, which no formula reads. Record B1 to F1 again.
3. Type 6 in A1, a precedent of B1, C1 and F1 but not of D1 or E1. Record B1 to F1 again.
4. Formulas > Calculate Workbook (or Calculate Now). Record B1 to F1 again.
5. If the web refuses the file or shows a repair or warning bar, record its text.

## Record

For each step, the five values (number, text, error or boolean) and any marker or tooltip the web shows on those
cells.

## What the engine will do (the ruling)

- Step 1: B1 42, C1 84, D1 7, E1 the text, F1 FALSE, with B1, D1 and E1 marked as values from the file.
- Step 2: all unchanged.
- Step 3: B1 `#NAME?`, so C1 `#NAME?` and F1 TRUE; D1 and E1 keep their values.
- Step 4: open; the web's answer decides.
