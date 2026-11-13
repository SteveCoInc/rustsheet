# Probe request `chart-insert`: what Excel for the web makes when a chart is inserted

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Asked by the charts
lane (poc-charts-cc): P-CH-8, P-CH-9 and P-CH-3 of `docs/ui/specs/charts/probes.md` on its branch
`cr/charts/docs-ui-freeze` (6cdb8d1f2, not on main when this was written), and the series half of P-CH-7.
Taskei P561803998. Nothing here has been run in Excel. It blocks no lane: every answer is the expected value of
an acceptance case that is pending until then.*

## What is asked

1. **The chart types the charts lane does not make yet** (P-CH-8): radar, bubble, stock, surface, pie of pie,
   bar of pie and 3-D. For each, what the saved chart part holds when Excel for the web makes one, and which of
   them Insert offers at all. With them, three defaults the lane's cases cite the same probe for: a chart's
   data table, a series on the secondary axis, and a filtered series.
2. **The histogram's automatic bins** (P-CH-9), for two data sets.
3. **Recommended Charts** (P-CH-3), for twelve selections.

Part 1 is a script, run once. Part 2 is by hand, on the workbook the script made. One workbook comes back.

## What Microsoft documents

Read on 2026-11-12. A sentence in quotation marks is the page's.

- **3-D charts cannot be made.** The service description of Excel for the web, "View 3D charts": "View 3D charts
  in your worksheet. You cannot create or edit 3D charts in Excel for the web."
  (https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online).
  So the six 3-D cases are expected to be refused, and are here to say how.
- **Which types Insert offers.** The same page, "Insert charts": "Choose from a variety of chart types, such as
  column, line, pie, or bar charts." No page I found lists them all for the web; Part 2 records the galleries.
- **Recommended Charts: two pages disagree.** The service description, "Recommended chart creation and editing
  with formatting controls": "Only Microsoft Excel desktop app supports advanced charts. Excel for the web does
  not support these features." Its metadata gives 2023-04-25 as its date. The support page "Create a chart with
  recommended charts" has a part for the web: "Recommended charts in Excel for the web creates interesting
  visuals about your data in a task pane.", then "Click Insert > Recommended Charts." and a choice between
  "+ Insert Pivot Chart" and "+ Insert Chart"
  (https://support.microsoft.com/en-us/excel/create-a-chart-with-recommended-charts). It also says "Recommended
  charts are available to Microsoft 365 subscribers in English, French, Spanish", and names Simplified Chinese
  and Japanese. So the web has a pane, not the desktop's dialog with an "All Charts" tab, and it may be absent
  for an account or a language. No page says how the recommendations are chosen or ordered.
- **The histogram's bins.** "Create a histogram", under Automatic: "This is the default setting for histograms.
  The bin width is calculated using Scott's normal reference rule."
  (https://support.microsoft.com/en-us/excel/create-a-histogram). The page shows the rule's formula as a
  picture, which I could not read. Its "Applies To" list names the desktop and mobile applications and not
  Excel for the web, and the fetch did not return what its Web tab holds. The rule in the statistics
  literature (Scott, 1979; not Microsoft's page) is a width of 3.49 times the standard deviation over the cube
  root of the count, often written with 3.5. Which constant and which standard deviation Excel uses is not
  said: "What Scott's rule gives" below has the four readings.
- **The API the script uses.** `Excel.ChartType` lists every type the script asks for, and the members it sets
  and reads are in the reference of `Excel.Chart`, `Excel.ChartSeries`, `Excel.ChartDataTable` and
  `Excel.ChartBinOptions` (https://learn.microsoft.com/en-us/javascript/api/excel/excel.charttype). I found no
  member for up and down bars, drop lines or high-low lines in the reference of `Excel.Chart` and
  `Excel.ChartSeries`: those are by hand.

## Part 1: the script

| Step | Do |
|---|---|
| 1 | Open a new blank workbook in Excel for the web |
| 2 | Open Script Lab in it and make a new snippet. Paste `cases.js` into the Script tab and this one line into the HTML tab: `<div id="cases"></div><pre id="out"></pre>` |
| 3 | Press Run, then click `RUN`. It adds 39 sheets, one after the other |
| 4 | When the line `"case":"END"` shows under the button, copy every line there into `lines.jsonl` |

The script logs an `ENV` line, one line a case and an `END` line. A chart the host refuses is that case's
answer, with the error's code, and the cases after it still run. In a workbook that already has a sheet named
as a case it writes nothing and logs one line.

## The chart cases

Each is a sheet named by the case, with the data at `A1` and the chart beside them. The chart's source is the
whole range, with the series by Excel's own choice of rows or columns.

| Case | `Excel.ChartType` | Range | Data | What its saved part answers |
|---|---|---|---|---|
| `Radar` | `Radar` | `A1:D6` | regions: three named series over five text categories | CH-TYPE-1: the group element, `c:radarStyle` |
| `RadarMarkers` | `RadarMarkers` | `A1:D6` | the same | the same, with markers |
| `RadarFilled` | `RadarFilled` | `A1:D6` | the same | the same, filled |
| `Bubble` | `Bubble` | `A1:C6` | X, Y and Size, five rows of numbers under a header | CH-TYPE-2: `c:bubbleScale`, `c:showNegBubbles`, which column is which |
| `Bubble3DEffect` | `Bubble3DEffect` | `A1:C6` | the same | `c:bubble3D` |
| `StockHLC` | `StockHLC` | `A1:D6` | five dates, then High, Low, Close | CH-TYPE-3: the series, `c:hiLowLines` |
| `StockOHLC` | `StockOHLC` | `A1:E6` | five dates, then Open, High, Low, Close | CH-TYPE-3: `c:hiLowLines`, `c:upDownBars` and their defaults; CH-ELEM-2 |
| `StockVHLC` | `StockVHLC` | `A1:E6` | five dates, then Volume, High, Low, Close | the bar group, the second axis pair |
| `StockVOHLC` | `StockVOHLC` | `A1:F6` | five dates, then Volume, Open, High, Low, Close | the same |
| `Surface` | `Surface` | `A1:D6` | regions | the group element, `c:wireframe`, the band formats |
| `SurfaceWireframe` | `SurfaceWireframe` | `A1:D6` | regions | the same |
| `SurfaceTopView` | `SurfaceTopView` | `A1:D6` | regions | `c:surfaceChart` against `c:surface3DChart`, `c:view3D` |
| `SurfaceTopViewWireframe` | `SurfaceTopViewWireframe` | `A1:D6` | regions | the same |
| `PieOfPie` | `PieOfPie` | `A1:B9` | eight items and their shares | CH-TYPE-4: `c:ofPieType`, `c:splitType`, `c:secondPieSize`, `c:serLines` |
| `BarOfPie` | `BarOfPie` | `A1:B9` | the same | the same |
| `3DColumnClustered` | `3DColumnClustered` | `A1:D6` | regions | whether it is made; if so `c:view3D` and the walls |
| `3DColumn` | `3DColumn` | `A1:D6` | regions | the same |
| `3DBarClustered` | `3DBarClustered` | `A1:D6` | regions | the same |
| `3DLine` | `3DLine` | `A1:D6` | regions | the same |
| `3DArea` | `3DArea` | `A1:D6` | regions | the same |
| `3DPie` | `3DPie` | `A1:B9` | eight items | the same |
| `Histogram1` | `Histogram` | `A1:A21` | the numbers 1 to 20 under a header | CH-TYPE-5: `cx:binning`; the line's `bins` |
| `Histogram2` | `Histogram` | `A1:A11` | 1, 1, 2, 3, 5, 8, 13, 21, 34, 55 under a header | the same |
| `DataTable` | `ColumnClustered` | `A1:D6` | regions; the chart's data table is then made visible | CH-ELEM-1: `c:dTable` and its defaults |
| `Secondary` | `ColumnClustered` | `A1:C6` | the regions' first two series; series 2 is then put on the secondary axis | CH-PANE-2: the second group and axis pair |
| `Filtered` | `ColumnClustered` | `A1:D6` | regions; series 2 is then filtered | CH-ELEM-3: `c15:filteredBarSeries`, or what is written instead |
| `Lines` | `Line` | `A1:C6` | the regions' first two series | nothing by the script: Part 2, step 5 |

A case's line also holds what the API reads back: the chart's type, and for the cases above with a default no
part states, the series' `bubbleScale`, `splitType`, `splitValue` and `secondPlotSize`, `gapWidth` and
`overlap`, the data table's five switches, and the histogram's `bins` (type, width, count, the overflow and
underflow switches and values). A property the host lacks is logged with its error code.

## The twelve selections

Data only, one sheet each, for Recommended Charts in Part 2. They are the charts lane's CH-REC-1 list.

| Case | Range | Data |
|---|---|---|
| `R01` | `A1:B5` | a text header row, text categories in A, numbers in B |
| `R02` | `A1:D5` | the same with three number columns |
| `R03` | `A1:B13` | a header row, the first of each month of 2026 in A (`mmm-yy`), numbers in B |
| `R04` | `A1:B5` | numbers in both columns, no header |
| `R05` | `A1:C5` | numbers in three columns, no header |
| `R06` | `A1:B4` | a header row, three items, shares 0.5, 0.3 and 0.2 |
| `R07` | `A1:B11` | a header row, ten items, values of mixed sign |
| `R08` | `A1:E6` | a header row, five dates (`d-mmm`), then Open, High, Low, Close |
| `R09` | `A1:C20` | a header row, nineteen rows: two number columns and a text column of three groups |
| `R10` | `A1:A10` | ten numbers, no header |
| `R11` | `A1:M2` | a row of twelve month names from B1, and a row of twelve numbers named in A2 |
| `R12` | `A1:B1000` | a header row, then 999 rows: an index and a value |

## Part 2: by hand, on the same workbook

Each picture is a PNG of the browser window at 100% browser zoom. Steps 1 and 2 matter most; 6 is the longest.
To select a range, click in the Name Box, type the range and press Enter.

| Step | Do | Return |
|---|---|---|
| 1 | Sheets `Histogram1` and `Histogram2`: a picture of each chart with its axis labels readable | `h1.png`, `h2.png` |
| 2 | Insert tab: open each chart gallery in turn and take a picture of each open gallery, with a type's name shown where hovering gives one | `gallery-<name>.png`, and in `RESULTS.md` every type's name as Excel shows it |
| 3 | Sheet `Radar`: select `A1:D6` and insert the plain radar chart from the galleries, if they have one. It lands beside the script's chart | whether there was one |
| 4 | For each case whose line says `"made":false`: on that case's sheet, select the range the table gives and look for the type in the galleries. Insert it if it is there | which were there |
| 5 | Sheet `Lines`: select the chart and look, in the Chart pane and in the menus, for high-low lines, drop lines and up/down bars. Turn on each one that is offered | which were offered, and where |
| 6 | Recommended Charts, for `R01`, `R02`, `R03` and `R08` first, then the other eight if time allows. On the sheet: select the case's range; Insert > Recommended Charts; take a picture of the pane, scrolling if it holds more than fits; then press `+ Insert Chart` on its first chart | `r01.png` and so on (`r01-2.png` for a second screen), and in `RESULTS.md` each pane's chart titles in order |
| 7 | Sheet `Filtered`: a picture of the chart. Say whether series `Beta` is drawn, whether a filter button shows beside the selected chart, and whether anything offers to hide a category. If it does, hide `East` | `filtered.png` |
| 8 | File > Save As > Download a Copy | `chart-insert.web.xlsx` |

If Insert has no Recommended Charts, say so and what it has in its place: that is the answer to step 6.

Whether a 3-D chart that a file already holds is drawn as 3-D is not asked here. `56557.xlsx` of the
`chart-save` request holds one on its chart sheet: if that workbook is opened by hand, a picture of that sheet
(`three-d.png`) answers it.

## What Scott's rule gives

My calculation, not Excel's: the width is the constant times the standard deviation over the cube root of the
count; the first bin starts at the least value and holds its upper edge; there are as many bins as reach the
greatest value. Excel's edges and counts, read from `h1.png` and `h2.png` and from the lines' `bins`, are held
against these rows.

| Case | Constant | Standard deviation | Width | Upper edges | Counts |
|---|---|---|---|---|---|
| `Histogram1` | 3.5 | sample | 7.63 | 8.63, 16.26, 23.88 | 8, 8, 4 |
| `Histogram1` | 3.5 | population | 7.44 | 8.44, 15.87, 23.31 | 8, 7, 5 |
| `Histogram1` | 3.49 | sample | 7.61 | 8.61, 16.21, 23.82 | 8, 8, 4 |
| `Histogram1` | 3.49 | population | 7.41 | 8.41, 15.83, 23.24 | 8, 7, 5 |
| `Histogram2` | 3.5 | sample | 28.91 | 29.91, 58.82 | 8, 2 |
| `Histogram2` | 3.5 | population | 27.43 | 28.43, 55.85 | 8, 2 |
| `Histogram2` | 3.49 | sample | 28.83 | 29.83, 58.65 | 8, 2 |
| `Histogram2` | 3.49 | population | 27.35 | 28.35, 55.70 | 8, 2 |

`Histogram1`'s counts tell the sample deviation from the population's (8, 8, 4 against 8, 7, 5), and the edges
tell 3.5 from 3.49 in the second decimal. `Histogram2` has the same counts in every reading, so only its edges
speak. A result that fits no row is the finding: then the pictures and the `bins` line are the record.

## What to return

`lines.jsonl`, `chart-insert.web.xlsx`, the pictures, and a `RESULTS.md` with the date, the browser, the build
of Excel for the web (File > About), the account's display language, and what the steps of Part 2 ask for in
words.

`node check.js` holds the script against a fake of Office.js: every case's sheet, data, type and range as the
tables above give them, a refused chart as its case's answer, a failed sync at each place in turn, and the
numbers of the table of readings.

## What the answers decide

| # | Observed | What it decides |
|---|---|---|
| 1 | Each type's saved group element and defaults | The expected values of CH-TYPE-1 to CH-TYPE-4 and of the `create` tests they name |
| 2 | Which 3-D types are refused, and how | Microsoft says they cannot be created. A refusal confirms that 3-D is a drawing task for files that have one, not an Insert one; a chart that is made says the documentation is behind |
| 3 | The galleries of step 2, and step 4 | The list CH-REC-2 shows, with each type's name as Excel gives it |
| 4 | `DataTable`, `Secondary`, `Filtered`, and steps 5 and 7 | The defaults CH-ELEM-1, CH-ELEM-2, CH-ELEM-3 and CH-PANE-2 expect, and whether the web offers lines and bars at all |
| 5 | The bins of `Histogram1` and `Histogram2` | CH-TYPE-5's expected bins, and which reading of Scott's rule the layout's bin test holds |
| 6 | The panes of step 6 | CH-REC-1's lists and their order; whether Recommended Charts is a pane with `+ Insert Chart`, as documented; what the first chart of each is, read from the saved workbook |
| 7 | The two charts of sheet `Radar`, the script's and step 3's | Whether a chart made through the API is the chart the ribbon makes, and so whether later chart probes may be scripted |

What Excel for the web makes is the record each case is compared with. Where the charts lane's chart then
differs, the lane changes it, or the difference is a known gap with a follow-up task (decision 0052).
