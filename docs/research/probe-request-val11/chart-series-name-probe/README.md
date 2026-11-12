# Probe request: a chart series whose reference is a defined name

*Design desk (poc-design-cc), 2026-11-12, for Taskei P561102112 (asked by the I-13 lane for the chart payload;
the three spellings of the chart formulas are the charts lane's draft). Nothing here has been run in Excel. The
request is for Excel for the web: no desktop Excel fills probes (the project lead's ruling of 2026-11-12). The
answers go into `RESULTS.md` beside this file.*

## What is asked

A chart series gives its name, its categories and its values each by a formula. Usually the formula is a range
of cells. Can it be a defined name, and what does Excel keep?

1. Does Excel plot a series whose values, categories or name are a defined name, of the workbook or of a sheet?
2. Does the chart part Excel saves keep the name or the cells the name was?
3. When the name is redefined, does the chart follow it?
4. When the name is deleted, what does the chart show, and what does the chart part hold?
5. Can a series be given a name through Office.js, and what does Excel then report as its source?

The collaborative sheet's chart payload stores a series reference as an anchored formula (I-13, I-02). Until
these are answered its planner treats a series reference that holds a name as it treats any formula that holds
one.

## What is known without Excel

- **A file desktop Excel saved keeps a workbook's name, spelled `[0]!Name`.** `dataValidationTableRange.xlsx` of
  the fetched `poi` suite (`docProps/app.xml`: Microsoft Excel, version 16.0300) has a bar chart of two series.
  Their categories are `<c:f>[0]!graph_names</c:f>` and their values `[0]!graph_values` and `[0]!graph_poi`,
  three names of the workbook, each with a cache of 38 points. That answers question 2 for a workbook's name on
  the desktop.
- **It is the only such chart on disk.** `python3 -I scan.py eval crates`, in a tree with the fetched suites,
  reads 1,671 workbooks with 74 chart parts. It finds a chart part by the relationship that links it, whatever
  the part is called, and a formula by its element in the chart namespace, whatever prefix the part gives it
  (review of CR-322108021, round 1: the first scan went by part names and by the `c:` prefix, and found the
  same 74 parts and the same formulas). Thirteen series formulas are not cells: the four above; six quoted
  texts such as `"Trades"`, a series' name given as a literal; one `All!#REF!`; and `standard` and
  `measurement` in `clone_sheet.xlsx`, bare words in a workbook that defines no name. None is a sheet's name.
  What the scan could not read, it says: 22 files are no readable packages (encrypted workbooks, and fuzzer,
  crash and other damaged cases among POI's test data), and one workbook's relationships are not XML. A tree with no fetched
  suite has 332 workbooks, 22 chart parts and no such formula.
- **Office.js documents a name as a series' data** since ExcelApi 1.21. `ChartSeries.setValues` takes "a `Range`
  or, in ApiSet 1.21, a string in en-US chart data-reference syntax. The string can contain a cell or table
  reference, a defined name, or a literal value or array" (`@types/office-js` 1.0.617). Whether Excel for the
  web has 1.21 is the first thing `SET` tells.
- Select Data, where a user types `=Sheet1!Sales` on the desktop, is a desktop dialog. What Excel for the web
  has in its place is asked by hand, below.

## The workbooks

`python3 -I generate.py <dir>` writes six workbooks. Each has one sheet, `Sheet1`, with these cells, three
names, and one clustered column chart of one series, with a legend:

| | A | B | C |
|---|---|---|---|
| 1 | Month | Value | Other |
| 2 | a | 10 | 1 |
| 3 | b | 20 | 2 |
| 4 | c | 15 | 3 |
| 5 | d | 25 | 4 |

`Label` is `Sheet1!$B$1`, `Months` is `Sheet1!$A$2:$A$5` and `Sales` is `Sheet1!$B$2:$B$5`. Column C is what
`Sales` is redefined to, and what no name names.

| File | The names are | The series' name, categories and values | sha256 |
|---|---|---|---|
| `series-control.xlsx` | the workbook's | `Sheet1!$B$1`, `Sheet1!$A$2:$A$5`, `Sheet1!$B$2:$B$5` | `43ac2ca854ee955059f687a8dfe0febec95f3cbcdd3b5b1a66c8862d8c14d3ee` |
| `series-workbook-name-values.xlsx` | the workbook's | `Sheet1!$B$1`, `Sheet1!$A$2:$A$5`, `[0]!Sales` | `e9f4a2988e8d433e56fa252b2fe7f13ea160e22acbba2a22ee452caffbeee69d` |
| `series-workbook-name.xlsx` | the workbook's | `[0]!Label`, `[0]!Months`, `[0]!Sales` | `273f2f1ec8f4041956d780625c396cd7493e6641d13978fa2586304417618967` |
| `series-sheet-name.xlsx` | Sheet1's | `Sheet1!Label`, `Sheet1!Months`, `Sheet1!Sales` | `6344c86b1d5535cd66c9791b2e049d4785b0ab203a0b110a1c3cd29ef0bb6c73` |
| `series-bare-name.xlsx` | the workbook's | `Label`, `Months`, `Sales` | `39d692b6408cedc2861ddc38f3f123e2ec7a70ddb1b93b80fed8be6417afa034` |
| `series-prefixed-workbook-name.xlsx` | the workbook's | `Sheet1!Label`, `Sheet1!Months`, `Sheet1!Sales` | `f36ea1ce58175d1abbe4e54ecceeada2973e887def0b657fdd460dd1f0315536` |

- The second file names the values alone, in the spelling of the desktop's file: if Excel refuses or repairs
  one of the others, this one says whether the values were the reason. The third spells all three so. The
  fourth gives the sheet its own names. The last two are spellings no file on disk shows Excel writing: a bare
  name, and a workbook's name after a sheet's name, which is how a user types it in Select Data.
- **Every cache in the chart parts is stale on purpose**: the series' name is cached as `stale`, its categories
  as `w`, `x`, `y`, `z` and its values as 901 to 904. So what the chart plots says whether Excel read the
  formula (`Value`; `a` to `d`; 10, 20, 15, 25) or kept the cache. The control tells what Excel does with a
  stale cache over plain cells, which is what the others are read against.
- The files were written by `generate.py`, not by Excel, and no Excel has opened them. The chart part has the
  shape of the one `tools/src/rustsheet_tools/pivot_chart.py` writes, which Excel for the web opens
  (`probes/t4130-pivot-chart-cache`, filled), with a legend added; the package has the parts of the hand-built
  fixtures Excel for the web has opened (`eval/suites/fixtures/cases/valueless-cell-forms.xlsx`).

## What to do

Run the control first, in both parts. If Excel refuses `series-control.xlsx`, stop there and send the error:
the package is then the suspect, not the names.

**First, through the oracle tool** (no Script Lab; it answers questions 1 and 3 of the table below from files).
For each of the six, with `<dir>` an absolute path (the command runs in `tools/`):

```
cd tools && uv run rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err
```

Please return, for each, the JSON output, or the `.err` file if the backend refused the workbook (a refusal is
an answer, beside an empty JSON), and the recalculated workbook, which the tool keeps as
`eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the file's sha256 above. Its chart part shows what Excel
for the web saved: the three formulas, and whether the stale caches were rewritten from the name's cells. The
tool's canary writer accepts all six (checked here, without the network).

**Then, in Excel for the web, with Script Lab.** Paste `cases.js` into the Script tab and this one line into
the HTML tab: `<div id="cases"></div><pre id="out"></pre>`. Press Run: the script puts three buttons on the
page, `READ`, `CHANGE` and `SET`. A click runs that case in the workbook that is open; its JSON line is logged,
and shown under the buttons to copy from. The cases run once each in a workbook, in that order: while a case
runs every button is off, and afterwards its own button and the ones before it stay off. Each new file needs
Script Lab opened in it again. For each file, in the table's order (the first four matter most; the last two
if there is time):

1. Upload the file and open it. Copy any message word for word, with its buttons and the one chosen. If Excel
   says a file is damaged or repairs it, that is the answer for that file: report it and go to the next.
2. Click `READ`. It reads the cells, the names and the series, then writes `read` into E1 so that Excel saves
   the workbook. If its line says there is no chart or no series, Excel dropped it: that is the answer for the
   file. Look at the chart: note the series' name in the legend, the category labels and the bars' heights.
3. File > Save As > Download a Copy, as `<file>-read.xlsx`.
4. Click `CHANGE`. It writes 11 into B2, then redefines `Sales` as `=Sheet1!$C$2:$C$5`, then deletes `Sales`,
   reading the names and the series after each. Look at the chart again and note the same three things.
5. Download a copy again, as `<file>-changed.xlsx`.

Then upload `series-control.xlsx` twice more, under other names. In the first:

6. Click `SET`. Before each attempt it sets the series to C2:C5, which reads 1, 2, 3, 4, so that an attempt
   Excel takes reads 10, 20, 15, 25 and one it ignores or refuses does not. It sets the series' values to the
   range of `Sales` (a Range object), then to the strings `Sales`, `=[0]!Sales`, `=Sheet1!Sales` and `=Sales`,
   and last its categories to `=Months`. Download a copy, as `series-control-set.xlsx`: it holds what the last
   attempts left. (`SET` refuses a workbook where `CHANGE` has run or whose names are the sheet's.)

In the second, where no case has run, by hand: select the chart and find where Excel for the web shows or
edits the series' data (a Select Data command, or the Data section of the chart's format pane). If a box holds
the series' values, type `=Sheet1!Sales` into it, and note whether it is taken and what the box shows
afterwards. If there is no such box, say so. If something was taken, download a copy, as
`series-control-hand.xlsx`.

**Return** the oracle tool's six outputs and workbooks, the thirteen JSON lines the script logged, the thirteen
or fourteen downloaded workbooks, and the notes. Each click logs one line; a call Excel refuses is in it, with
its error, where its answer would be.

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Is a series by name plotted from the name's cells, for each of the four spellings, and for the values alone? | the caches of the workbook the oracle tool returns; `READ`: `values:Values`, `values:Categories` and `name` against the stale caches; the control's line | which spellings the import of a chart accepts as a name |
| 2 | What does Excel report as the series' source? | `READ`: `source:*`, `sourceType:*` | what the payload shows a user for such a series |
| 3 | What does the saved chart part hold: the name, in which spelling, or the cells? Are the caches rewritten? | the workbook the oracle tool returns, and `<file>-read.xlsx`: `c:f` and the caches | what the export writes, and that the payload keeps the name |
| 4 | Does the chart follow the cells under the name? | `CHANGE`, step `cell` | that such a series is a dependent of the name's cells |
| 5 | Does the chart follow a redefined name? | `CHANGE`, step `redefine` | that the reference is the name, not its first value |
| 6 | What does the chart show after the name is deleted, and what does the part hold then? | `CHANGE`, step `delete`; `<file>-changed.xlsx` | what the payload does when a name it uses is deleted |
| 7 | Does a sheet's name behave as a workbook's? | the fourth file against the third | whether the payload needs the name's scope |
| 8 | Can the API give a series a name, in which spelling, and what is the source then? | `SET`: each attempt against the reset before it; `series-control-set.xlsx` | how the collaborative sheet writes such a series back to Excel |
| 9 | Does a series set to the range of a name keep the name? | `SET`, step `range of Sales` | that a Range drops the name |
| 10 | What does Excel for the web offer in place of Select Data? | the notes; `series-control-hand.xlsx` | what the sheet's own Select Data is compared with |

When the answers are in, they go to `RESULTS.md` here and to docs/excel-semantics/ as rules with this request as
their evidence. Where Excel for the web and the desktop's file disagree (the spelling `[0]!Name`), the entry
gives both, and the rule follows the web (decision 0039 rule 4).

## Checks, without Excel

- `python3 -I check.py` runs the generator and reads the six workbooks back: the cells, the names and their
  scope, the chart's three formulas and its stale caches, every part well-formed and every relationship's
  target present, and the sha256 of each as the table above gives it.
- `node check.js` runs the three cases against a fake of Office.js that is strict where a real host is (a
  property is readable only after a load that names it and a sync): one line a run whatever fails, a workbook
  without the chart left alone and described, the changes each case makes and their order, the reset before
  each of `SET`'s attempts, and a button a case in Script Lab.
- `cases.js` type-checks against `@types/office-js` 1.0.617 (`tsc --checkJs`).
- `scan.py` is the count under "What is known without Excel": counts and file names only. `python3 -I
  test_scan.py` holds it to the three names of a generated workbook under another prefix, as the default
  namespace, under another part name and through an absolute target, and to saying so when a linked chart
  part is missing or not XML, when relationships are not XML, and when a file is no package.
- Two readers that are not this generator load all six: this repository's importer gives back the fifteen
  cells and the three names with their scope (the engine draws no charts), and openpyxl 3.1.5 gives the same
  and the chart's three formulas and its caches. Neither is Excel.
- An independent read of the request before it went out found no part that departs from the schema (a
  validator calibrated on the Open XML SDK reports nothing in the chart part, with the legend) and nine things
  to fix in the script and the steps; they are fixed here.
