# Probe request `pivot-ui-2`: four more things about pivots in Excel for the web's own UI

> **val11 note:** `PivotTable-AllSources-input.xlsx` is a ClosedXML test file (MIT), so this change does not carry it: `python3 -I ../fetch_public_inputs.py pivot-ui-probe-2 .` downloads it at the revision the `pivot-corpus-closedxml` suite pins and checks its sha256.

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Automatic refresh, a
click on a pivot cell, date grouping, and pivots over sources the web cannot read. Asked by the pivot UI
lane (poc-pivot-ui-cc): requests P-7, P-9 (its UI part), P-6 (its UI part) and P-1 of
`docs/ui/specs/pivot/PROBES.md` on its branch `pvui/freeze-docs` (not on main when this was written). It follows
the request `pivot-ui-probe` (`docs/research/pivot-ui-probe`, not on main when this was written), which has
P-2, P-3 and P-4. Taskei P561753761. Nothing here has been run in
Excel, and no feature waits for it before the UI import lands.*

## What is asked, and what is not

Four things the pivot UI's cases rest on that the Graph oracle cannot show, because each is a gesture in
Excel's own UI. Microsoft documents part of each. Read on 2026-11-12:

- **"Refresh PivotTable data"** (https://support.microsoft.com/en-us/excel/refresh-pivottable-data):
  - "In newer versions of Excel, PivotTables can automatically refresh when connected to local workbook data."
    and "PivotTable Auto Refresh is currently available to participants of the Microsoft 365 Insider program.";
  - in its part for Excel for the web: "There are two ways to turn Auto Refresh on or off: through the
    PivotTable tab or the PivotTable Settings pane.", and, for a file's own pivots, a "Refresh data on file
    open" box in that pane;
  - "New PivotTables created from local workbook data will have Auto Refresh on by default." and "Auto Refresh
    is set per data source, so turning it on or off affects all PivotTables connected to that data source.",
    both in its part for Windows.

  So the setting exists on the web for some accounts. What this repository has measured is narrower than its
  entry's title ("PivotTable Auto Refresh is not stored or honoured by Excel for the web",
  `docs/excel-semantics/pivots.md`): a range written through Graph does not refresh a pivot, and the file the
  web saved then held no such setting. Part 1 asks what a typed edit does, with the setting off and on.
- **"GETPIVOTDATA function"** (https://support.microsoft.com/en-us/excel/functions/getpivotdata-function), which
  does not list Excel for the web among what it applies to: "You can turn this feature on or off by selecting
  any cell within an existing PivotTable, then go to the PivotTable Analyze tab > PivotTable > Options > uncheck
  the Generate GetPivotData option." Part 2 asks what the web writes and whether it has the option.
- **"Group or ungroup data in a PivotTable"**
  (https://support.microsoft.com/en-us/excel/get-started/group-or-ungroup-data-in-a-pivottable), which does not
  list Excel for the web either: "With time grouping, relationships across time-related fields are
  automatically detected and grouped together when you add rows of time fields to your PivotTables." and "In
  the Grouping box, select Starting at and Ending at checkboxes, and edit the values if needed." It does not
  name a number of days. Part 3 asks what the web does.
- **"Differences between using a workbook in the browser and in Excel"**
  (https://support.microsoft.com/en-us/excel/differences-between-using-a-workbook-in-the-browser-and-in-excel):
  "some kinds of data connections can prevent a workbook from being displayed in a browser window." Which, and
  what a pivot over one then shows, is Part 4.

## The workbooks

| File | What it is | sha256 |
|---|---|---|
| `t4130-pivot-auto-refresh.xlsx` | The workbook of the probe case `probes/t4130-pivot-auto-refresh`, byte for byte the file excel-graph calculated. `Data!A1:B4` is Region and Sales: East with an empty B2, West 20, East 5. Sheets `AR0` and `AR1` each hold a pivot over it (rows Region, Sum of Sales); `AR1`'s is saved to refresh on load. The two pivots are over the same range, each with its own cache. | `668bde32423065fc24e62f516357578db403333177d66cb3739c6255a2c6f90a` |
| `t4767-pivot-slicers.xlsx` | The workbook of `probes/t4767-pivot-slicers`, likewise; the same file as in `pivot-ui-probe`. Sheet `SA` has a pivot at A3: East 37, West 10, North 16 and a Grand Total of 63, in that order (A4:B7). | `ff5dfb4dae53bee4c75df7b1e100839b966a52786b6ee9d836f6868654de99e6` |
| `t4170-pivot-date-levels.xlsx` | The workbook of `probes/t4170-pivot-date-levels`, likewise. `Data!A1:B10` is Date and S: nine dates from 2018-12-15 to 2021-01-15, S a different power of two for each. Sheets `DC`, `DO`, `DT`, `DS`, `DY`, `DM` and `DX` each hold a pivot over `Data!A1:B10`, each with its own cache, in which Date is already grouped by months, quarters and years from 2019-01-01 to 2020-12-31; `Calc` holds GETPIVOTDATA formulas over them. | `1de4cf34ad068978de0b83b7085161899f5a96acd98a868f1b0fc8f40b56101b` |
| `PivotTable-AllSources-input.xlsx` | A workbook of ClosedXML's test data, saved by Excel 16.03, which the repository fetches as part of its `pivot-corpus-closedxml` suite. Seven sheets hold one pivot each, over a data connection, another workbook, a range, a table, a defined name, a scenario and a consolidation. | `0dfd174fbe668c27db490c5dfc1e7d8df8d075211424f5bd0b393bdc0a8ed73a` |

The first three are built with the repository's case builder, from its `tools` directory, and the same bytes
come out at every run:

    uv run rustsheet-spec2xlsx -o <name>.xlsx ../eval/suites/probes/cases/<name>.case.json

with `<name>` each of `t4130-pivot-auto-refresh`, `t4767-pivot-slicers` and `t4170-pivot-date-levels`. The
fourth is at
`eval/.cache/suites/pivot-corpus-closedxml/Other/PivotTable/Sources/PivotTable-AllSources-input.xlsx` after
`rustsheet-eval fetch`. A copy of each goes with this request.

## How

By hand, in Excel for the web. The four parts are independent, and each starts from a fresh copy of its file.
Please note the date, the browser and the build Excel reports (File > About), and whether the account is in
the Microsoft 365 Insider program, if you know. Write a sentence for each step, with a picture where a message
or a setting is asked. **Return** the notes as `RESULTS.md` beside this file, with the pictures and the
downloaded workbook of Part 1.

### Part 1: does a typed edit refresh a pivot (P-7; `t4130-pivot-auto-refresh.xlsx`)

| Step | Do | Write down |
|---|---|---|
| 1.1 | Open the workbook. Look at the pivots on `AR0` and `AR1`. | Their rows and numbers. excel-graph returned East 5, West 20 and a total of 25 on both. |
| 1.2 | Select the pivot on `AR0`. Look on the PivotTable tab for Auto Refresh, and in the PivotTable Settings pane for a setting that refreshes the pivot automatically, whatever it is called there (write its label down). | Is either there? Is it on or off for this pivot? The same for `AR1`. A picture of the pane. |
| 1.3 | Type 100 into `Data!B2` and press Enter. Go to `AR0` and `AR1` without pressing Refresh. | Their numbers: still 5 and 25, or East 105 and a total of 125? If they changed: at once, or after something else was done? |
| 1.4 | If 1.2 found the setting: change it for `AR0` alone (on if it was off, off if it was on) and look at `AR1`'s setting. Read both pivots' numbers, then type 200 into `Data!B2` and read them again. | The numbers right after the change, and after typing: East 205 and a total of 225 on a pivot whose setting is on would be an automatic refresh; 105 and 125 kept on one whose setting is off would be none. Did changing it for `AR0` change it for `AR1`? (The two are over the same range, each with its own cache.) |
| 1.5 | Add a sheet and copy `Data!A1:B4` to its A1. Insert a new pivot over the copy on a new sheet, rows Region, values Sales, and look at its setting before touching it. Type 300 into the copy's B2. Then insert one more pivot over `Data!A1:B4` itself and look at its setting. | Is automatic refresh on for a new pivot over data no other pivot uses, without being asked? Does it follow the edit (East 305, total 325)? Does the pivot over `Data!A1:B4` take the setting `AR0` and `AR1` have? |
| 1.6 | File > Save As > Download a Copy. | Return the file as `saved-auto-refresh.xlsx`. |

From `saved-auto-refresh.xlsx` we read whether the file now stores the setting, which it did not when the web
saved it after a Graph write.

### Part 2: what a click on a pivot cell writes in a formula (P-9; `t4767-pivot-slicers.xlsx`, sheet `SA`)

| Step | Do | Write down |
|---|---|---|
| 2.1 | Select D10, type `=`, click the cell that holds East's 37, press Enter. | The formula D10 now holds, character for character, and its value. |
| 2.2 | Look for an option named Generate GetPivotData: on the PivotTable tab, in its Settings pane, in the pivot's right-click menu. | Is it there, where, and is it on? |
| 2.3 | In D11, type `=`, click the cell that reads `East` (a label, not a value), press Enter. | The formula D11 holds. |
| 2.4 | If the option of 2.2 is there, turn it off, repeat 2.1 in D12 and 2.3 in D13, then turn it back on. | The formulas D12 and D13 hold. |

### Part 3: a date field dropped on rows, and the grouping box (P-6; `t4170-pivot-date-levels.xlsx`)

| Step | Do | Write down |
|---|---|---|
| 3.1 | Add a sheet and copy `Data!A1:B10` to its A1: the workbook's own pivots already group Date over `Data!A1:B10`, and a pivot over that range may take their grouping. Insert a new pivot over the copy on a new sheet. Read the field list before dragging anything. Then drag Date to Rows and S to Values. | The fields the field list shows before the drag (Date and S alone?) and after it (Date alone, or Years, Quarters, Months beside it), which of them are on Rows, and every row label of the report as it reads. A picture. |
| 3.2 | Right-click a date label of the report and choose Group. | Every box of the dialog, with what it holds: Starting at, Ending at, the list under By and which of its lines are selected, Number of days. If there is no Group command, say so and stop. |
| 3.3 | Select Days alone under By. Set Starting at to 2019-01-01, Ending at to 2020-12-31 and Number of days to 7, and confirm. | Is Number of days available once Days alone is selected? The row labels, word for word (the spans, and the labels for the dates before the start and after the end). |
| 3.4 | Open the dialog again and select Days and Months together. | Is Number of days still available? If it can be set, what happens on confirming: taken, refused, a message? |
| 3.5 | Ungroup. | The fields and labels afterwards. |

### Part 4: pivots over sources the web cannot read (P-1; `PivotTable-AllSources-input.xlsx`)

| Step | Do | Write down |
|---|---|---|
| 4.1 | Open the workbook. If it does not open, stop Part 4 here: the message is the answer, and one file per kind of source follows as a new request. | Does it open? Any bar or message about connections or links, word for word, and what it offers. |
| 4.2 | On each of the sheets `Connection - PT`, `External Workbook - PT`, `Scenario - PT` and `Consolidate - PT`: click a cell of the report. | Is the PivotTable tab shown? Is the field list shown, and does it list fields? |
| 4.3 | On each of those four: tick or drag a field in the field list, if there is one. | Is it taken? Any message. Undo it if it was. |
| 4.4 | On each of those four: look for Change Data Source and open it if it is there. Close it without changing the source. | Is it offered? What it shows for this pivot. |
| 4.5 | On each of those four, last, because a refresh that fails may change the report: press Refresh. | The result, and any message, word for word, and what the report reads afterwards. |
| 4.6 | On `Area - PT`, `Table - PT` and `Defined Name - PT`, whose data is in the workbook: the same three steps in the same order. | The same answers, as the control: these three should behave as any pivot does. |

No workbook of the fetched suites has a pivot over an OLAP cube or over the data model, so those two kinds are
not asked here.

## What the answers decide

The case numbers are the pivot UI lane's, from its PROBES file. For every row, what Excel for the web does
decides the case: where it differs from what a case proposes, the case is revised, and a difference that has to
stay is a known gap with a follow-up task, not a choice of the editor's (decision 0052).

| # | Question | Read from | Cases and entries |
|---|---|---|---|
| 1 | Does a typed edit refresh a pivot, with the setting off, on, and on a new pivot? | 1.2 to 1.5 | The record of where the editor differs from Excel (decision 0438 names this unverified); the entry "PivotTable Auto Refresh is not stored or honoured by Excel for the web" in pivots.md, whose title is then narrowed or confirmed |
| 2 | Does the file store the setting? | `saved-auto-refresh.xlsx` | the same entry |
| 3 | What does a click on a pivot's value or label write, and can it be turned into a plain reference? | 2.1 to 2.4 | PV-F01-03 |
| 4 | Does the web group a dropped date field by itself, and into which fields? | 3.1 | PV-F10-04 (an owner decision that should know what the web does) |
| 5 | What does the grouping box offer, and when is a number of days allowed? | 3.2 to 3.4 | PV-F10-01, PV-F10-02 |
| 6 | For a pivot over a connection, another workbook, a scenario or a consolidation: what shows, and what does each command say? | 4.1 to 4.5, against 4.6 | PV-F02-06 |

The pivot UI lane writes each answer into `docs/excel-semantics/pivots.md` with this request as its evidence
and updates its cases in the same change.
