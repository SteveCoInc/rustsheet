# Probe request `pivot-ui`: slicers, typed captions and what a slicer draws, in Excel for the web

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Asked by the pivot UI
lane (poc-pivot-ui-cc) for its freeze specs: requests P-2 (its UI parts), P-3 and P-4 of
`docs/ui/specs/pivot/PROBES.md` on its branch `pvui/freeze-docs` (b72a8aeba, not on main when this was written).
These three are the ones it builds from first; P-1, P-6, P-7 and P-9 follow in a second request. Nothing here
has been run in Excel, and no feature waits for it before the UI import lands.*

## What is asked, and what is not

The pivot UI's cases say what the editor does with a slicer on a table, with a deleted slicer, with a text
typed over a pivot's label, and how it draws a slicer and a timeline. Each rests on what Excel for the web
does, which the Graph oracle cannot show: it reads cells and saved parts, not a box drawn over them or a
message. Microsoft documents part of it. Read on 2026-11-12:

- **"Use slicers to filter data"**, its Web tab
  (https://support.microsoft.com/en-us/excel/get-started/use-slicers-to-filter-data):
  - "Only local PivotTable slicer creation is available in Excel for the web." and "To create slicers for
    tables, data model PivotTables, or Power BI PivotTables, please use Excel for Windows or Excel for Mac."
    So a table's slicer is not made on the web: Part 1 opens a file that has two;
  - a pivot's slicer is made with "On the PivotTable Analyze tab, select Slicer."; a style is chosen with "On
    the Slicer tab, select a colored style that you want."; one slicer serves several pivots through "Slicer
    Settings" and its "PivotTable Connections" section;
  - "To select more than one item in the Slicer box, hold Control, and then select the items that you want to
    show." and "To clear a slicer's filters, select Clear Filter in the slicer.";
  - a slicer is deleted with "Select the slicer, and then press Delete." or "Right-click the slicer, and then
    select Remove." What then becomes of the filter is not said;
  - the page names no setting for the header, the order, the columns or the items with no data.
- **"Differences between using a workbook in the browser and in Excel"**
  (https://support.microsoft.com/en-us/excel/differences-between-using-a-workbook-in-the-browser-and-in-excel):
  "Filters and slicers generally work in a browser window similar to how they work in Excel."
- **"Rename a field or item in a PivotTable or PivotChart"**
  (https://support.microsoft.com/en-us/excel/rename-a-field-or-item-in-a-pivottable-or-pivotchart), which does
  not list Excel for the web among what it applies to: "The following procedures do not change the Source Name
  of the field or item." It does not say what typing over a label in a cell does, nor what a name already in
  use gets.
- **"Create a PivotTable timeline to filter dates"**
  (https://support.microsoft.com/en-us/excel/create-a-pivottable-timeline-to-filter-dates), which does not list
  Excel for the web either: "four time levels (years, quarters, months, or days)". Part 4 opens a file that has
  timelines.

## The workbooks

| File | What it is | sha256 |
|---|---|---|
| `table-slicers.xlsx` | Built for this request from `table-slicers.case.json`: sheet `T` holds the table `Sales` over A1:D7 and two table slicers, on Region and on Product, that state no setting, so they keep Excel's defaults. No Excel has opened it. | `64f544613de8af87331e9f2d4a3a1485774267d0fa1585c237f1b38dff7e7a41` |
| `t4767-pivot-slicers.xlsx` | The workbook of the probe case `probes/t4767-pivot-slicers`, byte for byte the file excel-graph calculated (the sidecar's `input_sha256`). Each pivot sheet has a pivot at A3 (rows Region, Sum of Sales) and a slicer on Region. | `ff5dfb4dae53bee4c75df7b1e100839b966a52786b6ee9d836f6868654de99e6` |
| `t4767-pivot-timelines.xlsx` | The workbook of `probes/t4767-pivot-timelines`, likewise. Each pivot sheet has a pivot at A3 (rows Product, Sum of Sales) and a timeline on Date. | `ab7870a7595423ac64b58c24aa9542314cc7f87f51e62e5fef562006dfef3249` |

Build all three with the repository's case builder from its `tools` directory. Repeated builds use the same
bytes with unchanged builder code, locked dependencies and Python/zlib build; a different compression
implementation can change the ZIP bytes. Compare the generated SHA-256 values with the table before treating a
rebuild as byte-identical to an oracle input:

    uv run rustsheet-spec2xlsx -o table-slicers.xlsx ../docs/research/pivot-ui-probe/table-slicers.case.json
    uv run rustsheet-spec2xlsx -o t4767-pivot-slicers.xlsx ../eval/suites/probes/cases/t4767-pivot-slicers.case.json
    uv run rustsheet-spec2xlsx -o t4767-pivot-timelines.xlsx ../eval/suites/probes/cases/t4767-pivot-timelines.case.json

The data is the same in all: six records of Region, Product, Date and Sales, with Sales a different power of two
in each, so that a sum names its records.

| Region | Product | Sales |
|---|---|---|
| East | A | 1 |
| West | A | 2 |
| East | B | 4 |
| West | B | 8 |
| North | C | 16 |
| East | C | 32 |

So East is 37, North 16, West 10 and the total 63; North has product C alone, and West has no C.

## How

By hand, in Excel for the web. The four parts are independent, and each starts from a fresh copy of its file.
Please note the date, the browser, its zoom and the build Excel reports (File > About). Write a sentence for
each step and take a screenshot where a box, a mark or a message is asked, at 100% zoom of the sheet. **Return**
the notes as `RESULTS.md` beside this file, with the screenshots and the downloaded workbooks named in Parts 3
and 4.

### Part 1: a table's slicers (P-2; `table-slicers.xlsx`, sheet `T`)

| Step | Do | Write down |
|---|---|---|
| 1.1 | Open the workbook. | Does it open, with or without a message? Are two slicers drawn, Region and Product? If Excel refuses the file, stop here, say so, and run steps 1.2, 1.5 and 1.6 on sheet `TS` of `t4767-pivot-slicers.xlsx`, which has one table slicer, on Region. |
| 1.2 | Click a cell of the table and look through the tabs. | Is any command offered that makes a slicer for the table? (Microsoft: not on the web.) |
| 1.3 | Press North in the Region slicer. | The rows the table shows. The mark on the Region column's header. In the Product slicer: which buttons are drawn lighter, and the order of the three buttons now (North has C alone, so A and B have no data). A picture. |
| 1.4 | Clear the slicer. From the Product column's header, filter to A alone. | In the Region slicer: which buttons are drawn lighter, and the order of the three (North has no A). Is any button drawn as selected? A picture. |
| 1.5 | Clear the filter. From the Region column's header, set a text filter: begins with `E`. | What the Region slicer shows: East selected, nothing selected, or something else. Then press West in the slicer: which rows does the table show, and is the text filter still there? |
| 1.6 | Clear everything. Press East in the Region slicer, then select the slicer's box and press Delete. | Does the table stay filtered to East? What does the Region header's mark show? |

### Part 2: deleting a pivot's slicer, and a pivot that has one (P-2; `t4767-pivot-slicers.xlsx`)

| Step | Do | Write down |
|---|---|---|
| 2.1 | Open sheet `SL`. Its slicer has East alone selected. | The pivot's rows and numbers (excel-graph returned East 37 and a total of 37 for this sheet). |
| 2.2 | Select the slicer's box and press Delete. | The pivot's rows and numbers now: still East alone, or all three regions and 63? Is anything said? |
| 2.3 | Open sheet `SA`, where every item is selected. Select every cell of the pivot (from its first label to its total) and press Delete. | What goes and what stays: the pivot's cells, the slicer's box. If the slicer stays, press one of its buttons: what happens, and what is said? |

A pivot that also has a chart is not in this file; that half of the question is the engine lane's (T-5496).

### Part 3: text typed over a pivot's labels (P-3; `t4767-pivot-slicers.xlsx`, sheet `SA`)

Before the steps, drag Product to Columns in the field list, so that the pivot has row labels, column labels and
a total each way.

| Step | Do | Write down |
|---|---|---|
| 3.1 | Select the cell that reads `East`, type `Ost`, press Enter. | Is it taken? What the cell reads, what the slicer's button reads, and any message, word for word. |
| 3.2 | Type `All` over the row `Grand Total`. Type `Regions` over `Row Labels` and `Products` over `Column Labels`. | For each: taken, refused, or something else; any message. |
| 3.2a | Before changing or clearing any accepted caption, File > Save As > Download a Copy. | Return the file as `saved-captions.xlsx`; record which of 3.1 and 3.2 were accepted. |
| 3.3 | Type `North` over the cell that reads `West`: the label of another item of the same field. | What happens: a message, the two rows changing places, two rows that read `North`, or something else. |
| 3.4 | Undo if 3.3 changed anything. Type `NORTH` over `West`. | The same question for a text that differs from another item's only in its capitals. |
| 3.5 | Select the cell that reads `Ost` and press Delete. Then type `East` into it. | What each does: the old label back, an empty label, a refusal. |
| 3.6 | File > Save As > Download a Copy. | Return the final file as `saved-caption-reset.xlsx`. |

From `saved-captions.xlsx` we inspect the accepted captions from 3.1 and 3.2 in the pivot definition (`item@n`,
`grandTotalCaption`, `rowHeaderCaption`, `colHeaderCaption`) and compare the cache items with the input.
`saved-caption-reset.xlsx` records the final state after the collision and reset tests; it is not evidence of
an earlier caption that those tests replaced.

### Part 4: what a slicer and a timeline draw (P-4; both `t4767` workbooks)

Every step is a picture at 100% zoom, with the browser's zoom at 100%. If the Slicer tab shows the box's height
and width, write them down with their unit.

| Step | Do | Picture of |
|---|---|---|
| 4.1 | `t4767-pivot-slicers.xlsx`, sheet `SA`. | The Region slicer as it opens: every item selected, default settings. |
| 4.2 | Click the pivot, PivotTable Analyze > Slicer, tick Product. In the new slicer press C. | Both slicers. West has no C: how is its button drawn in the Region slicer, and where does it stand among the three? |
| 4.3 | Select the Region slicer and open Slicer Settings on the Slicer tab. | The pane. Write down every setting it offers, word for word. |
| 4.4 | For each setting of 4.3 that the pane offers, change that one alone and put it back after: the header off; the order descending; hide items with no data; do not mark items with no data; items with no data not last; two columns. | The Region slicer under each. Say which of the six the web does not offer. |
| 4.5 | Sheet `SM`. Its slicer was saved with the header off, descending order, two columns, the style Light 2, East and West selected, North marked as having no data, and the marking of such items turned off. | The slicer as it opens: this shows the settings of 4.4 that the web's pane lacks. |
| 4.6 | Sheet `SA` again. Select the Region slicer and choose one light style. Take its picture and use File > Save As > Download a Copy before changing the style; return `saved-slicer-light.xlsx`. Then choose one dark style, take its picture and download `saved-slicer-dark.xlsx`. | The Region slicer under each style, with the style's name as its tip gives it and the matching downloaded filename. |
| 4.7 | `t4767-pivot-timelines.xlsx`, sheet `TL`. | The timeline as it opens, with its selection (the first quarter of 2019). |
| 4.8 | Change the timeline's level to years, to quarters, to months and to days, and select one period at each. | The timeline at each of the four levels. Say if a level cannot be chosen on the web. |

From `saved-slicer-light.xlsx` and `saved-slicer-dark.xlsx` we read the selected style names and any explicit
style definitions Excel writes. The corresponding screenshots record the rendered appearance. If the files
contain only built-in style names, the element definitions remain unverified by this probe; do not infer them
from the names alone.

## What the answers decide

The case numbers are the pivot UI lane's, from its PROBES file. For every row, what Excel for the web does
decides the case: where it differs from what a case proposes, the case is revised, and a difference that has to
stay is a known gap with a follow-up task, not a choice of the editor's (decision 0052).

| # | Question | Read from | Cases |
|---|---|---|---|
| 1 | Does the web open a table with slicers at their defaults, and does it offer to make one? | 1.1, 1.2 | PV-F04-01 |
| 2 | How are a table slicer's items with no data drawn and ordered? | 1.3, 1.4 | PV-F04-04 |
| 3 | What does a slicer show beside a custom filter on its own column, and what does a press do to that filter? | 1.5 | PV-F04-07 |
| 4 | Does deleting a slicer take its filter away, for a table and for a pivot? | 1.6, 2.2 | PV-F04-12 (a table), PV-F03-05 (a pivot) |
| 5 | What is left of a slicer when its pivot is deleted? | 2.3 | PV-F03-14 |
| 6 | Which labels of a pivot take a typed text, and what does a text that is another item's get? | 3.1 to 3.5 | PV-F06-01, PV-F06-02 (which labels), PV-F06-04, PV-F06-06 |
| 7 | How is a typed text stored? | `saved-captions.xlsx` | PV-F06-04 |
| 8 | What does a slicer draw under each setting and under a light and a dark style? | 4.1 to 4.6, `saved-slicer-light.xlsx`, `saved-slicer-dark.xlsx` | PV-F07-02, PV-F07-04; PV-F05-07 for the two styles |
| 9 | What does a timeline draw at each level? | 4.7, 4.8 | PV-F05-07 |

The pivot UI lane writes each answer into `docs/excel-semantics/pivots.md` with this request as its evidence
and updates its cases in the same change. Which case each row changes is that lane's own reading (its mail of
2026-11-12, on its branch at 82182a9b4), which added two cases for questions 4 and 5. PV-F03-09 and PV-F03-10,
which its PROBES file lists under P-2, are file cases of the engine lane's T-4962 and are not asked here.
