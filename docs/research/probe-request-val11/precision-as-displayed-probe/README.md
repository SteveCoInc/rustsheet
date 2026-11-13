# Probe request `precision-as-displayed`: what "Set precision as displayed" rounds, and when

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For Taskei P559132978:
the engine ignores `calcPr fullPrecision="0"`, and its owner asked for a short design and Excel probes before
any code. The research note beside this directory (`../precision-as-displayed-2026-11-11.md`) has what Microsoft
documents (sections S1 and S2), what LibreOffice does (S4) and the questions the documentation leaves open (S6).
A provisional rule below is what Microsoft documents where it documents anything, and says so.*

## Why these cannot be static cases

A case of `eval/suites/probes` is a workbook and its values after one full recalculation, and the case builder
writes no `calcPr fullPrecision` (`tools/`, no setting for it). Most questions here are about a sequence: turn
the option on, change a format, read, calculate, read. Office.js has the option as
`Workbook.usePrecisionAsDisplayed` (ExcelApi 1.9, read and write), and `Range.values` returns what a cell
stores, so Script Lab can ask all of them.

## How

- **In Script Lab:** paste `cases.js` into the Script tab and this one line into the HTML tab:
  `<div id="cases"></div><pre id="out"></pre>`. Press Run: the script puts one button a case on the page.
  Click one button only: it runs that case in the workbook that is open, which must be new and empty (`READ`
  alone runs in an opened file), and the next case needs a new workbook with Script Lab opened in it again.
  The case's JSON line is logged, and shown under the buttons to copy from. The click turns the buttons off.
- **Office.js:** run `cases.js`, one case per **new, empty** workbook: `run("A")`, `run("E")`, `run("F")`,
  `run("X")`, `run("FMT")`, `run("O")`, `run("S")`, `run("K")`, `run("W")`, `run("G")`, `run("CF")`, `run("I")`,
  `run("ST")`, `run("IT")`. Each logs one JSON line with the build, the platform, the calculation mode and the
  option as it found them (`modeAtStart`, `padAtStart`). Every read logs the option, the mode, the application's
  `calculationState`, and for each cell its value, its kind (`valueTypes`: Double or Integer, String, Boolean,
  Error, Empty, which JSON alone cannot tell apart for an error and a text that spells it), its text, its formula
  and its number format. Please return the lines as `results.jsonl`; it is committed beside this README with a
  `RESULTS.md`, as the earlier probe directories do.
- **IT changes iterative calculation**, which is the application's setting on the desktop, not the workbook's.
  `run()` logs the three settings it found (`iterationAtStart`) and puts them back when the case ends or fails.
- **The option is destructive.** Microsoft: "Data will permanently lose accuracy when switching this property
  from `false` to `true`" (research note, S1). So `run()` refuses a workbook in which any sheet holds a value
  (`WorkbookNotEmpty`) and changes nothing in it: no cell, no setting, not the calculation mode. When a case
  ends or fails, `run()` puts the option and the calculation mode back as it found them, and writes neither
  unless the case had changed it. Please do not run it beside real data all the same.
- `node check.js` runs every case against a fake of Office.js that keeps a grid, the option and the mode. It
  checks that the script holds exactly the cases named here and the 99 rows listed below; one JSON line per
  case; the option, the mode and the iteration settings as they were found afterwards, from a host in Automatic
  and in Manual mode and one with iterative calculation on, also when any one sync fails (except the last sync,
  which is the one that restores, failing with its writes dropped); that a workbook with values, an unknown case
  and a failed first sync change nothing at all; the shape of every array written; that the table cases write
  the rows below where this README says and that every table read holds, under each column letter, that column's
  cells; and that a cell read as -0 leaves other evidence than one read as 0. It needs no Excel and says nothing
  about Excel's values.
- `python3 -I check.py` reads the two table files `generate.py` writes back against the rows listed below: the
  constant and the formula of each row in its format, the readers in General, the array over F2:F3, no cached
  formula value, and `fullPrecision` in the one file only.
- Excel for the web is the set of record: excel-graph is the only oracle (decision 0266 rule 2), and Script Lab
  runs there. Whether the property can be set in Excel for the web is the first thing the run tells (a refused
  write is logged with its error code). If the web refuses it, no desktop Excel will set it instead (the
  project lead's ruling of 2026-11-12; decision 0266 rule 2), and the generated files through the oracle tool
  say what follows (decision 0462). If Excel for the web honours a file's flag, the cases that need the option
  on and never write it are asked in a generated workbook that has it, as READ is, and only what needs the
  option written has no oracle: those rules are entered as `NO-ORACLE` on Microsoft's documentation
  (docs/excel-semantics/README.md). If it ignores the flag, Excel for the web and the files desktop Excel
  saved disagree, which is `CONFLICTING`, and the question goes back to the project lead.
- The script writes numbers with `Range.values` and formulas with `Range.formulas`. They stand in for typing.
  Where an answer differs when it is typed, say so in `RESULTS.md`.
- A case whose requirement set the host lacks should say so instead of failing silently (the script logs the
  error code).

## A and E: which formats round, and to what

`cases.js` holds one table `T` of 99 rows: a number format, a number, and a formula that gives that number. Row
*i* is sheet row *i* + 1. Column A holds the number as a constant and column B the formula, both in the row's
format. Columns C and D read A and B (`=A2`, `=B2`), in General, set after the formula so that the reader does
not take the format of the cell it reads.

| Case | Steps | Question |
|---|---|---|
| A | write the table; read; turn the option on; read; full calculation; read; full calculation; read | For each row: what does the cell store once the option is on (`values`), what does it show (`text`), and what does a reader get? Does a second calculation move a number again? |
| E | turn the option on; A and B: the format, then the number; E and F: the number in a General cell, then the format; read after each step; full calculation; read; full calculation; read | Is a number rounded when it is entered into a formatted cell? Is a constant rounded when its cell is formatted afterwards? |

What the rows ask, with our provisional rule. "Rounds as shown" means: the cell stores the number its text
shows.

| Rows (format) | Question | Our provisional rule |
|---|---|---|
| `0.00`, `0.0`, `0`, `#,##0.00`, `000`, `00.0` | Rounds as shown? Ties (2.675, 0.125, 0.5, 2.5, -0.5) and doubles just under a tie (1.005, 8.325, 0.285, 0.49999999999999994): as ROUND gives, or as the binary value falls? | rounds as shown, as Microsoft documents ("to whatever format, including decimal places, is displayed"); a tie as the cell's text shows it |
| `0` with 1.2345678901234567E+19 and 123456789012345.6 | Is a number shown with 15 digits and zeros stored as those digits? | yes, the text's number |
| 16 and 18 decimals with `=0.1+0.2`, `=1/3` | Does a format with more decimals than a double holds change the bits (0.30000000000000004 to 0.3)? | UNVERIFIED: yes if the rule is the text's number; no rule in the documentation |
| `General` | Unchanged? | UNVERIFIED: unchanged. The documentation speaks of formats with decimal places; LibreOffice leaves General alone unless its decimals are limited |
| `0%`, `0.0%`, `0.00%` | 0.12345 in `0.0%` stores 0.123? | rounds as shown |
| `0.00E+00`, `0E+0`, `0.0E+0`, `##0.0E+0` | 12345 in `0.00E+00` stores 12300? | rounds as shown (LibreOffice: the same) |
| `# ?/?`, `# ??/??`, `# ?/8`, `# ?/2`, `?/?` | 0.3 in `# ?/?` stores 2/7? 0.04 stores 0? | UNVERIFIED: the fraction's value (LibreOffice: the same) |
| `#,##0,`, `0.0,,`, `0.00,"K"` | 1234567 in `#,##0,` stores 1235000? | UNVERIFIED: rounds as shown (LibreOffice: the same) |
| dates and times | 45000.75 in `m/d/yyyy` stays 45000.75, or becomes 45000? 0.5004 in `h:mm`? | UNVERIFIED: unchanged (LibreOffice leaves dates and times alone) |
| `@` | In A the cell is Text before it is written, so the constant and the formula's own text are both stored as text: a control. In E, columns E and F get the Text format after the number: unchanged? | unchanged |
| two sections, a condition | The section the number selects decides: -1.2345 in `0.00;-0.0` stores -1.2? | UNVERIFIED: yes |
| `[>=100]0;0.00` with 99.999, `[<1.46]0.0;0` with 1.455 | A number whose rounding belongs to the other section: 99.999 shows 100.00, and 100 shows 100; 1.455 shows 1.5, and 1.5 shows 2. What is stored, what is shown afterwards, and does a second calculation move it again (1.5 to 2)? | UNVERIFIED, and no rule proposed: the section is chosen by the number before rounding, so the stored number can show another text than the one it was rounded to (research note, S7.1) |
| `0.00;;`, `;;;`, `"abc"`, `#,###` with 0.4, `#.##` with 0.001 | A section that shows no digit, or nothing at all: unchanged, or 0? | UNVERIFIED: unchanged where the section has no digit placeholder; 0 where its placeholders show none |
| `0.0;-0.0;"zero"` with 0.04 and -0.04 | Rounds to 0 by the first or second section, and then shows `zero`: is 0 stored? Is -0.04 read back as 0 or as -0 ("Signed zero" below)? | UNVERIFIED: 0 |
| literals, colours, padding, a locale tag, the accounting format | Rounds as shown, whatever else the section prints? | rounds as shown |
| `#.##`, `0.??`, `?.?`, `#` | Optional digits round as `0` digits do? | rounds as shown |

## The rows of the table

Every row of `T`, as `cases.js` holds it; `node check.js` fails when the two differ. The number is the
JavaScript double the script writes as the constant; the formula is what it writes beside it.

| Sheet row | Format | Number | Formula |
|---|---|---|---|
| 2 | `0.00` | 1.2345 | `=1.2345` |
| 3 | `0.00` | 2.675 | `=2.675` |
| 4 | `0.00` | 1.005 | `=1.005` |
| 5 | `0.00` | -1.005 | `=-1.005` |
| 6 | `0.00` | 0.125 | `=0.125` |
| 7 | `0.00` | 0.135 | `=0.135` |
| 8 | `0.00` | 0.285 | `=0.285` |
| 9 | `0.00` | 8.325 | `=8.325` |
| 10 | `0.00` | 1234567.891 | `=1234567.891` |
| 11 | `0.00` | 0.004 | `=0.004` |
| 12 | `0.00` | -0.004 | `=-0.004` |
| 13 | `0.00` | 123456789012.345 | `=123456789012.345` |
| 14 | `#,##0.00` | 1234.5678 | `=1234.5678` |
| 15 | `0.0` | 0.25 | `=0.25` |
| 16 | `0.0` | 0.35 | `=0.35` |
| 17 | `0.0` | -0.25 | `=-0.25` |
| 18 | `0.0` | 0.05 | `=0.05` |
| 19 | `0` | 0.5 | `=0.5` |
| 20 | `0` | 1.5 | `=1.5` |
| 21 | `0` | 2.5 | `=2.5` |
| 22 | `0` | -0.5 | `=-0.5` |
| 23 | `0` | -2.5 | `=-2.5` |
| 24 | `0` | 0.49999999999999994 | `=0.49999999999999994` |
| 25 | `0` | -0.001 | `=-0.001` |
| 26 | `0.0` | -0.04 | `=-0.04` |
| 27 | `0` | 12345678901234567000 | `=12345678901234567000` |
| 28 | `0` | 123456789012345.6 | `=123456789012345.6` |
| 29 | `000` | 1.5 | `=1.5` |
| 30 | `00.0` | 1.25 | `=1.25` |
| 31 | `0.0000000000000000` | 0.30000000000000004 | `=0.1+0.2` |
| 32 | `0.000000000000000000` | 0.3333333333333333 | `=1/3` |
| 33 | `0.00000000000000` | 0.30000000000000004 | `=0.1+0.2` |
| 34 | `General` | 1.23456789012345 | `=1.23456789012345` |
| 35 | `General` | 0.30000000000000004 | `=0.1+0.2` |
| 36 | `General` | 123456789.123456 | `=123456789.123456` |
| 37 | `General` | 12345678901234567000 | `=12345678901234567000` |
| 38 | `General` | 0.000001234567890123 | `=0.000001234567890123` |
| 39 | `0%` | 0.125 | `=0.125` |
| 40 | `0%` | 0.135 | `=0.135` |
| 41 | `0%` | 0.12345 | `=0.12345` |
| 42 | `0.0%` | 0.12345 | `=0.12345` |
| 43 | `0.00%` | 0.123456 | `=0.123456` |
| 44 | `0.00E+00` | 12345 | `=12345` |
| 45 | `0.00E+00` | 0.00012345 | `=0.00012345` |
| 46 | `0.00E+00` | -98765 | `=-98765` |
| 47 | `0E+0` | 15000 | `=15000` |
| 48 | `0.0E+0` | 0.000999 | `=0.000999` |
| 49 | `##0.0E+0` | 12345 | `=12345` |
| 50 | `##0.0E+0` | 123456 | `=123456` |
| 51 | `# ?/?` | 0.3 | `=0.3` |
| 52 | `# ?/?` | 2.375 | `=2.375` |
| 53 | `# ?/?` | -0.3 | `=-0.3` |
| 54 | `# ?/?` | 0.04 | `=0.04` |
| 55 | `# ??/??` | 0.3 | `=0.3` |
| 56 | `# ??/??` | 3.14159 | `=3.14159` |
| 57 | `# ?/8` | 0.3 | `=0.3` |
| 58 | `# ?/2` | 0.3 | `=0.3` |
| 59 | `?/?` | 2.375 | `=2.375` |
| 60 | `#,##0,` | 1234567 | `=1234567` |
| 61 | `#,##0,` | 499 | `=499` |
| 62 | `#,##0,` | 500 | `=500` |
| 63 | `0.0,,` | 123456789 | `=123456789` |
| 64 | `0.00,"K"` | 1234567 | `=1234567` |
| 65 | `m/d/yyyy` | 45000.75 | `=45000.75` |
| 66 | `yyyy` | 45000.75 | `=45000.75` |
| 67 | `d-mmm-yy` | 45000.75 | `=45000.75` |
| 68 | `m/d/yyyy h:mm` | 45000.123456 | `=45000.123456` |
| 69 | `h:mm` | 0.5004 | `=0.5004` |
| 70 | `h:mm:ss` | 0.50001 | `=0.50001` |
| 71 | `[h]:mm` | 1.5004 | `=1.5004` |
| 72 | `mm:ss.0` | 0.000012 | `=0.000012` |
| 73 | `@` | 1.2345 | `=1.2345` |
| 74 | `0.00;-0.0` | 1.2345 | `=1.2345` |
| 75 | `0.00;-0.0` | -1.2345 | `=-1.2345` |
| 76 | `0.0;(0.00)` | -1.2345 | `=-1.2345` |
| 77 | `[>=100]0;0.00` | 150.6 | `=150.6` |
| 78 | `[>=100]0;0.00` | 50.678 | `=50.678` |
| 79 | `[>=100]0;0.00` | 99.999 | `=99.999` |
| 80 | `[<1.46]0.0;0` | 1.455 | `=1.455` |
| 81 | `0.00;;` | -1.2345 | `=-1.2345` |
| 82 | `0.00;;` | 0 | `=0` |
| 83 | `;;;` | 1.2345 | `=1.2345` |
| 84 | `0.0;-0.0;"zero"` | 0.04 | `=0.04` |
| 85 | `0.0;-0.0;"zero"` | -0.04 | `=-0.04` |
| 86 | `0.0;-0.0;0.000` | 0 | `=0` |
| 87 | `0.0" kg"` | 1.2345 | `=1.2345` |
| 88 | `"abc"` | 1.2345 | `=1.2345` |
| 89 | `[Red]0.0` | 1.2345 | `=1.2345` |
| 90 | `[$-409]0.00` | 1.2345 | `=1.2345` |
| 91 | `$#,##0.00_);[Red]($#,##0.00)` | -1234.5678 | `=-1234.5678` |
| 92 | `_($* #,##0.00_);_($* (#,##0.00);_($* "-"??_);_(@_)` | 0.004 | `=0.004` |
| 93 | `_($* #,##0.00_);_($* (#,##0.00);_($* "-"??_);_(@_)` | 1234.5678 | `=1234.5678` |
| 94 | `#.##` | 1.2345 | `=1.2345` |
| 95 | `#.##` | 0.001 | `=0.001` |
| 96 | `0.??` | 1.5 | `=1.5` |
| 97 | `0.??` | 1.2345 | `=1.2345` |
| 98 | `#,###` | 0.4 | `=0.4` |
| 99 | `#` | 0.4 | `=0.4` |
| 100 | `?.?` | 0.04 | `=0.04` |

## Signed zero

Several rows ask whether a negative number that rounds to zero is stored as 0 or as -0 (`0.00` with -0.004, `0`
with -0.001, `0.0` with -0.04, `0.0;-0.0;"zero"` with -0.04). JSON writes both as `0`, so every read also lists
the cells whose value the API gave as -0: `negativeZeros`, by sheet row in a table read and by `[row, column]`
within the range elsewhere.

A cell that is listed was read as -0. A cell that is not listed was read as 0 through Office.js, which is not
yet a statement about what Excel stores: the API may give 0 for both. Case K's two controls say whether a read
can show -0 at all. If neither control is listed, the stored sign stays unanswered by this script, and
`RESULTS.md` should say so instead of recording 0.

## The other Office.js cases

| Case | Steps | Question | Our provisional rule |
|---|---|---|---|
| F | A1 = 1.23456 and B1 `=1.23456`, both `0.000`; C1 `=A1*1000`, D1 `=B1*1000`; option on; read; F9; read; A1:B1 to `0.00`; read; F9; read; full; read; A1:B1 to `0.0000`; read; F9; read; full; read | Does a format change alone round a constant, and a formula's value? At once, at F9, or at a full calculation? After `0.00`, is B1 1.23 (calculated again from 1.23456) or 1.24 (its stored 1.235 rounded again)? After `0.0000`, is A1 still 1.24 or 1.235 (digits gone) and B1 1.2346 (calculated again)? Do C1 and D1 follow at once? | UNVERIFIED. A constant: rounded at the format change, and its digits are gone ("permanently"). A formula: calculated again and rounded, with its readers, at the format change |
| X | as F, in Manual mode from before the option is turned on | Is a constant rounded at once in Manual mode? Are formulas and readers stale until F9? | UNVERIFIED: constants at once; formula cells and readers at F9 |
| FMT | the option off throughout; B1 = 1.5, B2 `=B1*2`, F1 `=RAND()`; read twice; then one change at a time, each followed by a read: B1's number format, B2's, the number format of the empty cells B3:D3, B1's fill colour; then B1 = 2; read; F9; read | The control for F and X: with the option off, does a change of format alone calculate anything? F1 changes at every recalculation and at nothing else, so it says which steps calculated. (T-5187 asks this of excel-graph; this is the same question through Office.js.) | UNVERIFIED: no format change calculates; the value edit and F9 do |
| O | A1 = 1.23456 and B1 `=1.23456` in `0.00`, C1 `=A1*1000`, D1 `=B1*1000`; option on; read; option off; read; F9; read; full; read | With the option off again: A1 stays 1.23 (documented). Is B1 1.23456 again at once, at F9, or only at a full calculation? | A1 stays 1.23 ("the original underlying values cannot be restored"). B1: UNVERIFIED, at once |
| S | A1 `=SEQUENCE(4)/3` spilling over A1:A4 with formats `0.0`, `0.00`, General, `0%`; C1 `=SUM(A1#)`, C2 `=A2*3`, C3 `=SUM(A1:A4)` | Is each spilled cell rounded by its own format? Do `A1#` and `A1:A4` give the same sum? | UNVERIFIED: each cell by its own format; the same sum |
| K | `0.00` cells holding TRUE, the text `1.2345`, `=1/0`, nothing, `="1.2345"`, `=1.2345>1`; readers `=A1*1` and so on; and two controls for signed zero, A7 `=0*-1` and A8 = -0 written through the API | Only numbers are rounded: is the text still `1.2345` to its reader? Can a read through the API show -0 at all? | only numbers; the controls have no rule |
| W | option on; into `0.00` cells: a number, text Excel reads as a number, the same through `formulas`, a paste of values from a General cell, a paste of the cell with its General format; then a paste of formats (`0.00`) over a General constant and a General formula | Is every way of writing a number rounded? Does a paste that brings its own format keep the digits? Does a format that arrives by a paste of formats round a constant, and a formula's value? | UNVERIFIED: rounded by the format the cell has after the write; a paste of formats is a format change (F) |
| G | General cells in a narrow column (`columnWidth = 40`), holding 123456789.123456 (constant and formula) and 0.1234567890123 | Does the option follow what a narrow General cell shows? | UNVERIFIED: no, General is unchanged |
| CF | A1 = 1.23456 and A2 `=1.23456`, General, with a conditional format (`=TRUE`) whose number format is `0.0`; A3 = 1.23456 and A4 `=1.23456` with a rule that reads its own cell (`=A3>1.2`, format `0`) | Does the option follow the format the cell shows, or the cell's own? If the shown one: a cell shown as 1 no longer meets its own rule; what does Excel store, and does it settle? | UNVERIFIED: the cell's own format, so the rule changes nothing |
| I | A1 `=1/3` and A2 = 0.3333333 in `0.00`; B1 `=A1*3`, B2 `=A2*3` (General); C1 `=1/3*3` in `0.00`; C2 `=SUM(A1:A2)*3` | Is only what a cell stores rounded? B1 and B2 are 0.99 if they read 0.33; C1 is 1 unless a value inside a formula is rounded too | only stored values: 0.99, 0.99, 1 and 1.98 |
| ST | A1 = 1.23456 and B1 `=1.23456` with a named style `PadProbe` (General); C1 `=A1*1000`, D1 `=B1*1000`; option on; the style's number format set to `0.00`; read; F9; read; full; read | Does a format that changes through a named style round the cells that have the style, as a direct format change does (F)? | UNVERIFIED: yes, the same as F |
| IT | iterative calculation on, 3 iterations, no early stop; option on; A1 `=A1+1/3` in `0.00`, B1 `=B1+1/3` in General (the control); read; F9; read; F9; read | Is a cell on a loop rounded at each iteration (0.33, 0.66, 0.99) or when the calculation ends (1)? | UNVERIFIED: at each iteration, since each iteration stores the cell |

## By hand

- **SAVE.** `run("SAVE")` in a new workbook builds A1 = 1.23456 (`0.00`), B1 `=1.23456` (`0.00`), C1 `=A1*1000`,
  turns the option on and **leaves it on**. Save the workbook as .xlsx and unzip it. What do `xl/workbook.xml`'s
  `calcPr` (`fullPrecision`, any other attribute) and the `<v>` of A1, B1 and C1 hold? Provisional rule:
  `fullPrecision="0"`, and 1.23, 1.23 and 1230.
- **READ.** `python3 -I generate.py <dir>` writes two files that say the option is on while their cells hold
  more digits than their formats show, as a writer other than Excel can leave them: A1 = 1.23456 (`0.00`), B1
  `=A1*1000` cached 1234.56, C1 `=1.23456` (`0.00`) cached 1.23456, D1 `=C1*1000` cached 1234.56. Open each in a
  new Excel session and call `run("READ")`, which reads A1:D1, calculates (F9), reads, calculates fully and
  reads, and writes nothing else.
  - `pad-unrounded.xlsx` (sha256 `4aee54c9af451ee6bf060bcf84c673322ac823c58e33bfc2c9f5979e94f76263`).
  - `pad-unrounded-fullcalc.xlsx` (sha256 `63bea3d49a19502aa546c61518bfad359b21700d0b7a24f62e8713959fd5f24a`),
    the same with `fullCalcOnLoad="1"`.

  Does Excel round the constant A1 when it loads the file, at a calculation, or never? Are the cached formula
  values kept until something calculates? Provisional rule: UNVERIFIED. The design assumes the worst case for a
  reader of files: Excel does not round a constant on load, so a file's constants are taken as they are. The
  same two files can go through excel-graph for Excel for the web's values after its full calculation.

## Through excel-graph

`generate.py` also writes the table as a file, so that the oracle of record can answer its static half with no
Script Lab:

- `pad-table.xlsx` (sha256 `bffb99f7e72f0b16daa5fd0f2092b570bc8b1739fcef256a6fbbd5f10a442734`): every row of the
  table above, the number as a constant in column A and its formula in column B, both in the row's format, C and
  D reading them in General, and one Ctrl+Shift+Enter array over F2:F3 (`=A2:A3/7`) whose first cell is in
  `0.00` and whose second is in General. `calcPr fullPrecision="0"`. The formula cells hold no cached value.
- `pad-table-control.xlsx` (sha256 `9d133794e6e66d933dd6ed04e67a7db14c7af7d8405b2e60beec0545856ba896`): the same
  without `fullPrecision`.

For each, with `<dir>` an absolute path (the command runs in `tools/`), and the control first:

```
cd tools && uv run rustsheet-oracle recalc -b excel-graph <dir>/pad-table-control.xlsx \
  > <dir>/pad-table-control.graph.json 2> <dir>/pad-table-control.graph.err
```

The tool's canary writer accepts all four generated files (checked here, without the network). A refusal is an
error in the `.err` file beside an empty JSON, and it is an answer. If Excel refuses the control, stop there:
nothing here has opened a workbook of this generator in Excel yet, so the package is then the suspect, not the
option. If `recalc` refuses a file with the option on for a failed canary while the control passes, that too is
an answer, not a failure: Excel changed a General cell (the canary's A4 is `SQRT(2)` to the bit), and the error
text lists what the canary cells held. Please return the JSON and `.err` files, and the recalculated workbooks,
which the tool keeps as `eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the sha256 given above for each
file (only for a file whose canary passed: on a failed canary the tool deletes the workbook, and the `.err`
file is all there is): their `<v>` in column A say whether Excel rewrote a constant.

What it answers, by Excel for the web's full calculation of a file: what a formula cell stores under each format
(B, and its reader D), what a reader of an unrounded constant gets (C: does Excel round a file's constants?),
and whether each member of a Ctrl+Shift+Enter array follows its own format (F2:F3). The array divides by 7,
so that its members have more digits than `0.00` shows whether or not Excel has rounded A2 and A3: F3 with two
decimals says a member follows the anchor's format, F3 with every digit and F2 with two decimals says each
follows its own, and every digit in both says neither is rounded. What it cannot ask is a sequence, so E, F, X,
FMT, O, W, ST and IT stay with the script, and so do S, K, G, CF and I, which the file does not hold. If Excel
for the web refuses the Office.js property, this is the only set from the oracle of record.

## What the answers decide

SPEC-0002 and SPEC-0028 treat a cell's number format as invisible to calculation (T-5185 plans to skip the
recalculation of a format-only edit). With the option on, a format is an input of every numeric cell. The design
(research note, S7) needs to know: the rounding function (A, G, CF), when a constant loses its digits (E, F, W),
whether a format change is a recalculation (F, X), what turning the option off owes (O), whether a spilled cell
follows its own format (S), what a file may hold (SAVE, READ), what is rounded and where a format comes from (I,
IT, ST), and whether a format change calculates at all with the option off (FMT).
