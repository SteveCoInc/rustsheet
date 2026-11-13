# Probe request `chart-save`: what a save by Excel for the web does to a chart nobody touched

> **val11 note:** the 22 input workbooks are public test files (calipers, MIT; Apache POI, Apache-2.0; LibreOffice, MPL-2.0), so this change does not carry them. Run `python3 -I ../fetch_public_inputs.py chart-save-probe inputs` from this directory: it downloads each from its pinned revision and checks it against `inputs/SHA256SUMS`. The Graph route's `record_graph.py` is main's `docs/research/chart-selection-probe/record_graph.py` (the identical copy this request carried was dropped); `save_graph.py` here is the recorder that ran it.

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Asked by the charts
lane (poc-charts-cc): P-CH-1 of `docs/ui/specs/charts/probes.md` on its branch `cr/charts/docs-ui-freeze`
(6cdb8d1f2, not on main when this was written). Taskei P561803998. Nothing here has been run in Excel for the
web. It blocks no lane: it says whether one acceptance case of the charts lane is Excel's behaviour or stricter
than Excel.*

## What is asked

One question. A workbook holds a chart. Somebody changes a cell the chart does not read, and Excel for the web
saves. Does the chart's part come back byte for byte? If not, what changed: the series' ids alone, the form of
the XML, or more?

The charts lane's own save keeps an untouched chart part, its style part and its colours part byte for byte
(D-CHART-8, `apps/collab-sheet/decisions/`). Its acceptance case CH-FILE-2 says so, and cites this probe for
whether Excel does the same.

## What Microsoft documents

Nothing about the bytes of a part. I read the service description of Excel for the web
(https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online)
and "Differences between using a workbook in the browser and in Excel"
(https://support.microsoft.com/en-us/excel/differences-between-using-a-workbook-in-the-browser-and-in-excel) on
2026-11-12. The first says charts can be made and the second that they are shown; neither says what a save
writes. So there is no documented rule to hold the answer against, and the readings below are mine.

## What is on disk already

The charts lane's corpus (`crates/rustsheet-charts/tests/corpus/calipers/` on poc/charts) holds three workbooks
twice, an `init.xlsx` and a `golden.xlsx`, both saved by Excel 16 (the goldens by Excel for Windows 16.0 build
20326, their `meta.json`). `compare.py` over the three pairs, the init as "before":

| Workbook | Sheet | Chart | Chart part | Style part | Colours part | Other parts | Frame |
|---|---|---|---|---|---|---|---|
| `roundtrip-scatter_chart.xlsx` | Chart1 | 1 | ids only (1 of 1 changed) | identical | identical | none | identical |
| `roundtrip-scatter_chart.xlsx` | Sheet1 | 1 | ids only (1 of 1 changed) | identical | identical | none | identical |
| `roundtrip-three_charts.xlsx` | Sheet2 | 1 | ids only (2 of 2 changed) | none | none | none | identical |
| `roundtrip-three_charts.xlsx` | Sheet2 | 2 | ids only (1 of 1 changed) | none | none | none | identical |
| `roundtrip-three_charts.xlsx` | Sheet3 | 1 | ids only (2 of 2 changed) | none | none | none | identical |
| `roundtrip-with_chart.xlsx` | Sheet2 | 1 | ids only (2 of 2 changed) | none | none | none | identical |

An id is a series' `c16:uniqueId`. In every one the first eight digits, the series' index, are the same and the
rest is new: `{00000000-2E68-4865-930A-7AF8D75A82D0}` against `{00000000-9935-429A-AA0E-E02151169607}`.

What this shows: two saves by Excel 16 of one workbook give chart parts that are equal byte for byte except the
series' ids. What it does not show: that Excel replaces ids a part already has. Neither file of a pair need be
the other's ancestor. `roundtrip-with_chart`'s init was last saved on 2026-09-10 and its golden on 2026-09-08,
from another folder, and both carry a creation date of 2008. If both were saved from a source with no ids, each
save drew its own.

## The readings the answer tells apart

| Reading | An input saved by Excel 16, with ids | An input not in Excel's form |
|---|---|---|
| A. Excel for the web keeps the text of a part it did not edit | identical | identical |
| B. It keeps the text and draws new series ids | ids only | identical, or differs by the ids it adds |
| C. It writes the part again from its model | identical, ids only, or differs where its writer is not Excel 16's | differs: the form becomes Excel's |

The inputs saved by Excel 16 tell A from B. The inputs not in Excel's form tell A and B from C: an input whose
part is indented, or has its attributes in another order, comes back as it was only if the text is kept.

## The inputs

Twenty-two workbooks of the charts lane's corpus, unchanged. They are not on main when this is written; the
copies to use are fetched into `inputs/` by `python3 -I ../fetch_public_inputs.py chart-save-probe inputs` (from their public repositories at pinned revisions; this change does not carry them), and `inputs/SHA256SUMS`
holds the lines below.

| Input | Written by | Charts | Edit on sheet |
|---|---|---|---|
| `officejs-chart_column.golden.xlsx` † and the nine other `officejs-chart_*.golden.xlsx` | Excel for Windows 16.0, after an Office.js script | one each: area, axes, bar, column, doughnut, legend, line, pie, scatter, title, each with a style part and a colours part | `Sheet1` |
| `default-chart_titles.golden.xlsx`, `scratch-chart.golden.xlsx` | the same | one each | `Sheet1` |
| `roundtrip-three_charts.golden.xlsx` † | Excel for Windows 16.0 | three, on two sheets | `Sheet1` |
| `roundtrip-with_chart.golden.xlsx` | the same | one | `Sheet1` |
| `roundtrip-scatter_chart.golden.xlsx` † | the same | two: one on a sheet, one on a chart sheet | `Sheet1` |
| `WithThreeCharts.xlsx` | Excel 2007: no series ids, no style parts | three | `Sheet1` |
| `chartTitle_withTitle.xlsx` | Excel 2013: no series ids | one | `Sheet1` |
| `56557.xlsx` † | not Excel's form: attributes in alphabetical order, no `standalone` in the XML declaration | one, on a chart sheet | `Data` |
| `123233_charts.xlsx` | not Excel's form: indented XML, one element to a line; a sheet part named `Sheet1.xml` whose relationships are named `sheet1.xml.rels` | four, on one sheet | `Page1_1` |
| `64450.xlsx` | not Excel's form: no application properties, a carriage return alone after the XML declaration | one, in a workbook that also holds a PivotTable | `Hoja 1` |
| `SimpleHistogram.xlsx` † | Excel for the web ("Microsoft Excel Online") | one Excel 2016 chart (`chartEx`), a histogram | `Sheet1` |
| `waterfall.xlsx` | Excel 16 | one Excel 2016 chart, a waterfall | `Sheet1` |

† The five to run if the workbooks are opened by hand.

```
78d2bbf337c356546a2031bea155b47b591532f43d7cd00ef56614a1e14271ce  officejs-chart_area.golden.xlsx
4a7120bb305bddb65c3abbf8292aadf17f30963ce10384e0c786ff5d017d2747  officejs-chart_axes.golden.xlsx
5c6c3d18bd57bf2b8bfe36c1d88487691b8903d3464ff8c581bcd5dcdff01f81  officejs-chart_bar.golden.xlsx
13b723530c3d0f1c4d5d7bc6c6ba6f70679ca444412fa78bf7ca4914c2fe881a  officejs-chart_column.golden.xlsx
888494be596c6d235ce0fa3f1019687ad2bf55ff9391f8dfec74e172ae89dbf2  officejs-chart_doughnut.golden.xlsx
de7e762be6e3dd82e54353db470f29b7b650bd4373c7e00cb8e503e709d764de  officejs-chart_legend.golden.xlsx
23ae12cde7d0a5ff6a01e53c02002e31e3bb591d3cdd95c794e200fe58e75b85  officejs-chart_line.golden.xlsx
5924d46b92f46a4c8325ca815cc0a4013445d56b0cbe48d6a21098d5d0fc7845  officejs-chart_pie.golden.xlsx
edabd6a959b7bd6d26b713c98cdf059d3e038d59fb88c02f2329360c952f318f  officejs-chart_scatter.golden.xlsx
0bf5422f8007e13ef037ab260a05d72fb6bc371ac69c166c96a81a7b38bbfbc0  officejs-chart_title.golden.xlsx
ca2e068a18f530bb85a17e352341e60e02cbf459443e8763615c4d450d6471d9  default-chart_titles.golden.xlsx
e083c7b3ff8c0a0fc6d8d027a281a845e01b93d59d12226c8d88ec4b011a22b0  scratch-chart.golden.xlsx
2ad2eabe3fb91f2599d9a83630b7d52af745c7b160a93155d9cc42421f48412f  roundtrip-three_charts.golden.xlsx
3a3acf795f1f1b88a73e4d9385a797c51c8bf216572b718a84b17fbe8df300d2  roundtrip-with_chart.golden.xlsx
4613b5c9faa7f47cca160a361544730a8efef16618c237ebfdd63a97537a66bd  roundtrip-scatter_chart.golden.xlsx
629b69bd250d9b4e173eb2088eb80b656b7748b0287cd7b0df0c2dc22c6af7b0  WithThreeCharts.xlsx
586c174285621c081020da523c4a814a9ad62b75d86f49276faa1b6036919395  chartTitle_withTitle.xlsx
1660ad90dfb9e81d4b3fb6bf35d8a24d9515fdbe7abcfebaa909da08f1c01f2d  56557.xlsx
47e9e8dea87494d17dbacce9aa3a9ec31cb2b26b288de05ca95c0a4622ca7f80  123233_charts.xlsx
203eaf3275f3d4edaf000728defcece4bc44306852f27b59c9371e52d2338dec  64450.xlsx
dfe40e742cf92aad408273d17bb0733e1571c309678ec3a3148e10e45a946e56  SimpleHistogram.xlsx
f23626e52b75c7e57a6b3a6c69361bdacdbf2648a77c198963daacc4cba2e660  waterfall.xlsx
```

In every one, cell `AZ200` of the sheet named in the last column is empty, and no chart reads it.

## The steps

The edit is the same for every input: the number 1 into `AZ200` of the sheet the table names.

**Through Microsoft Graph**, all twenty-two. `docs/research/chart-selection-probe/record_graph.py` recorded an
earlier probe this way, and its `session_run` is the pattern, with the input's bytes uploaded where it uploads a
blank workbook:

| Step | Call |
|---|---|
| 1 | Upload the input unchanged |
| 2 | `POST .../workbook/createSession` with `{"persistChanges": true}` |
| 3 | `PATCH .../workbook/worksheets('<sheet>')/range(address='AZ200')` with `{"values": [[1]]}`, in the session |
| 4 | `POST .../workbook/closeSession`, wait until the item's eTag changes, download the content |

**By hand**, the five marked †:

| Step | Do |
|---|---|
| 1 | Upload the input to OneDrive and open it in Excel for the web, for editing |
| 2 | Click the tab of the sheet the table names (`Sheet1`; `Data` in `56557.xlsx`, whose active sheet is its chart sheet). Click in the Name Box, left of the formula bar, type `AZ200`, press Enter |
| 3 | Type `1`, press Enter, and wait until the title bar says the workbook is saved |
| 4 | File > Save As > Download a Copy |

An input that does not open, or opens for viewing only, is an answer: record the message, word for word. A
step that Graph refuses is one too: record the status and the error's text.

If both routes are at hand, please run `officejs-chart_column.golden.xlsx` by both. That tells whether a save
through Graph is the browser's save.

## What to return

- Each saved workbook, named as its input with `.web.xlsx` for `.xlsx`.
- `RESULTS.md`: the route, the date, the build of Excel for the web where the route shows one (File > About),
  what did not open, and what this prints:

  ```
  python3 -I compare.py <the directory of inputs> <the directory of saved workbooks>
  ```

`compare.py` finds a chart from its sheet (sheet, drawing, frame, chart part), so a part the save renamed is
still compared with itself. `python3 -I check.py` holds it against small packages built for the purpose.

## What the answers decide

| # | Observed | What it decides |
|---|---|---|
| 1 | The inputs saved by Excel 16: identical, ids only, or differs | Whether CH-FILE-2 ("an untouched chart saves back as it was read", byte for byte) is Excel for the web's behaviour, or stricter than it by the ids, or by more |
| 2 | Ids only | A series' `c16:uniqueId` does not survive a save by Excel. `crates/rustsheet-charts/src/model/series.rs` on poc/charts calls it "Excel's stable series id", and the collaborative store takes it as the series' own id at import: the entry in `docs/excel-semantics/charts.md` then says what the id is stable across |
| 3 | The three inputs not in Excel's form | Reading A or B against C: whether Excel keeps a part's text or writes its model. Under C the list of what it writes differently is the record for CH-FILE-5 ("an edited chart saves its edit and nothing else") |
| 4 | The style and colours columns | Whether Excel keeps those parts as they were; CH-FILE-2 says the lane does |
| 5 | `SimpleHistogram.xlsx`, `waterfall.xlsx` | CH-FILE-3: whether an Excel 2016 chart's part is kept byte for byte |
| 6 | `roundtrip-scatter_chart.golden.xlsx`, `56557.xlsx`: whether they open for editing | A first answer to P-CH-2 (chart sheets), before its own request |
| 7 | `64450.xlsx`, `123233_charts.xlsx`: whether they open | Whether Excel for the web reads a package whose part names differ in case from its relationship parts' names, and one with no application properties |
| 8 | The column input by both routes | Whether later chart probes may be recorded through Graph |

What Excel for the web writes is the record CH-FILE-2 is compared with. Where the lane's save then differs, the
lane changes it, or the difference is a known gap with a follow-up task (decision 0052).
