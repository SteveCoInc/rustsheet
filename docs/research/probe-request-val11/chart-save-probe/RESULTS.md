# chart-save probe: results (Excel for the web, through Microsoft Graph)

- **Route:** Microsoft Graph, all 22 inputs, unattended, on 2026-11-13. `save_graph.py` (beside this file) does the
  README's four Graph steps: upload unchanged, `createSession` with `persistChanges: true`, `PATCH` 1 into `AZ200` of
  the named sheet, `closeSession`, wait for the eTag to change, download. Graph shows no build number. No input was run
  by hand, so question 8 (Graph against the browser's own save) stays open.
- **Inputs:** fetched by `python3 -I ../fetch_public_inputs.py chart-save-probe inputs`, every sha256 checked. The
  saved `*.web.xlsx` copies are not committed: they are derived from the third-party inputs, which this change also
  does not carry. Rerun `save_graph.py` to get them back.
- **Did not open:** `123233_charts.xlsx`. `createSession` gave 501 `FileCorruptTryRepair` (inner
  `unsupportedWorkbook`: "The workbook contains unsupported features or exceeds the size limit."). So question 7: the
  web refuses the package whose sheet part is `Sheet1.xml` with relationships `sheet1.xml.rels`. `64450.xlsx` (no
  application properties) opened and saved.

## What it answers

1. **Inputs saved by Excel 16: identical.** Every chart, style and colours part came back byte for byte, and the
   existing series ids were kept, not redrawn. The two exceptions are the pie and the doughnut. Their parts differ
   only by a `c16:uniqueId` that the save adds to each `c:dPt` (data point) that had none. So CH-FILE-2 is Excel's
   behaviour for a part already in Excel's form (reading A).
2. Series ids survive a save. A data point's id is added where it is missing.
3. **Inputs not in Excel's form: rewritten (reading C).** `56557.xlsx`, `64450.xlsx`, `WithThreeCharts.xlsx` (Excel
   2007) and `chartTitle_withTitle.xlsx` (Excel 2013) come back in Excel's form:
   - attribute order and the `standalone` declaration;
   - the `c16r2` namespace;
   - `c:date1904`, `c:roundedCorners`, `c14:style` through `mc:AlternateContent`;
   - series ids;
   - empty `c:layout` elements dropped.

   The graphic frames of these four differ too.
4. Style and colours parts: identical wherever the input had them.
5. Excel 2016 charts: `SimpleHistogram.xlsx` (saved by Excel Online before) came back identical, apart from its frame.
   `waterfall.xlsx` came back rewritten, with `cx:txPr` blocks added, and its frame differs.
6. Chart sheets: `roundtrip-scatter_chart.golden.xlsx` and `56557.xlsx` both opened for editing and saved.

## `python3 -I compare.py inputs <saved>`

| Workbook | Sheet | Chart | Chart part | Style part | Colours part | Other parts | Frame |
|---|---|---|---|---|---|---|---|
| `123233_charts.xlsx` | not returned | | | | | | |
| `56557.xlsx` | Chart | 1 | differs | identical | identical | none | differs |
| `64450.xlsx` | Hoja 2 | 1 | differs | none | none | none | differs |
| `SimpleHistogram.xlsx` | Sheet1 | 1 | identical | identical | identical | none | differs |
| `WithThreeCharts.xlsx` | Sheet2 | 1 | differs | none | none | none | differs |
| `WithThreeCharts.xlsx` | Sheet2 | 2 | differs | none | none | none | differs |
| `WithThreeCharts.xlsx` | Sheet3 | 1 | differs | none | none | none | differs |
| `chartTitle_withTitle.xlsx` | Sheet1 | 1 | differs | identical | identical | none | differs |
| `default-chart_titles.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_area.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_axes.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_bar.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_column.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_doughnut.golden.xlsx` | Sheet1 | 1 | differs | identical | identical | none | identical |
| `officejs-chart_legend.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_line.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_pie.golden.xlsx` | Sheet1 | 1 | differs | identical | identical | none | identical |
| `officejs-chart_scatter.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `officejs-chart_title.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `roundtrip-scatter_chart.golden.xlsx` | Chart1 | 1 | identical | identical | identical | none | identical |
| `roundtrip-scatter_chart.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `roundtrip-three_charts.golden.xlsx` | Sheet2 | 1 | identical | none | none | none | identical |
| `roundtrip-three_charts.golden.xlsx` | Sheet2 | 2 | identical | none | none | none | identical |
| `roundtrip-three_charts.golden.xlsx` | Sheet3 | 1 | identical | none | none | none | identical |
| `roundtrip-with_chart.golden.xlsx` | Sheet2 | 1 | identical | none | none | none | identical |
| `scratch-chart.golden.xlsx` | Sheet1 | 1 | identical | identical | identical | none | identical |
| `waterfall.xlsx` | Sheet1 | 1 | differs | identical | identical | none | differs |

Where a chart part differs (one element to a line, `-` before the save, `+` after):

`56557.xlsx`, Chart, chart 1 (`xl/charts/chart1.xml`):
    -<?xml version="1.0" encoding="UTF-8"?>
    -<c:chartSpace xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    +<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    +<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:c16r2="http://schemas.microsoft.com/office/drawing/2015/06/chart">
    -<c:layout/>
    -<a:bodyPr anchor="ctr" anchorCtr="1" rot="0" spcFirstLastPara="1" vert="horz" vertOverflow="ellipsis" wrap="square"/>
    +<a:bodyPr rot="0" spcFirstLastPara="1" vertOverflow="ellipsis" vert="horz" wrap="square" anchor="ctr" anchorCtr="1"/>
    -<a:defRPr b="0" baseline="0" i="0" kern="1200" spc="0" strike="noStrike" sz="1400" u="none">
    +<a:defRPr sz="1400" b="0" i="0" u="none" strike="noStrike" kern="1200" spc="0" baseline="0">
    -<c:layout/>
    -<a:bodyPr anchor="ctr" anchorCtr="1" rot="0" spcFirstLastPara="1" vert="horz" vertOverflow="ellipsis" wrap="square"/>
    +<a:bodyPr rot="0" spcFirstLastPara="1" vertOverflow="ellipsis" vert="horz" wrap="square" anchor="ctr" anchorCtr="1"/>
    ... and 4 more lines
`64450.xlsx`, Hoja 2, chart 1 (`xl/charts/chart1.xml`):
    -<c:chartSpace xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:m
    +<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:c16r2="http://schemas.microsoft.com/office/drawing/2015/06/chart">
    +<c:date1904 val="0"/>
    +<c:lang val="en-US"/>
    +<c:roundedCorners val="1"/>
    +<c:style val="2"/>
    +<c:autoTitleDeleted val="1"/>
    +<c:grouping val="clustered"/>
    +<c:varyColors val="1"/>
    +<c:strCache>
    +<c:ptCount val="1"/>
    +<c:pt idx="0">
    ... and 293 more lines
`WithThreeCharts.xlsx`, Sheet2, chart 1 (`xl/charts/chart1.xml`):
    -<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    +<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:c16r2="http://schemas.microsoft.com/office/drawing/2015/06/chart">
    +<c:date1904 val="0"/>
    +<c:roundedCorners val="0"/>
    +<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
    +<mc:Choice Requires="c14" xmlns:c14="http://schemas.microsoft.com/office/drawing/2007/8/2/chart">
    +<c14:style val="102"/>
    +</mc:Choice>
    +<mc:Fallback>
    +<c:style val="2"/>
    +</mc:Fallback>
    +</mc:AlternateContent>
    ... and 33 more lines
`WithThreeCharts.xlsx`, Sheet2, chart 2 (`xl/charts/chart2.xml`):
    -<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    +<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:c16r2="http://schemas.microsoft.com/office/drawing/2015/06/chart">
    +<c:date1904 val="0"/>
    +<c:roundedCorners val="0"/>
    +<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
    +<mc:Choice Requires="c14" xmlns:c14="http://schemas.microsoft.com/office/drawing/2007/8/2/chart">
    +<c14:style val="102"/>
    +</mc:Choice>
    +<mc:Fallback>
    +<c:style val="2"/>
    +</mc:Fallback>
    +</mc:AlternateContent>
    ... and 21 more lines
`WithThreeCharts.xlsx`, Sheet3, chart 1 (`xl/charts/chart3.xml`):
    -<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    +<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:c16r2="http://schemas.microsoft.com/office/drawing/2015/06/chart">
    +<c:date1904 val="0"/>
    +<c:roundedCorners val="0"/>
    +<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
    +<mc:Choice Requires="c14" xmlns:c14="http://schemas.microsoft.com/office/drawing/2007/8/2/chart">
    +<c14:style val="118"/>
    +</mc:Choice>
    +<mc:Fallback>
    +</mc:Fallback>
    +</mc:AlternateContent>
    -<c:layout/>
    ... and 31 more lines
`chartTitle_withTitle.xlsx`, Sheet1, chart 1 (`xl/charts/chart1.xml`):
    -<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    +<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:c16r2="http://schemas.microsoft.com/office/drawing/2015/06/chart">
    -<c:layout/>
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    +<c16:uniqueId val="{00000000-3B32-4D9B-9246-1DBFDE19B9CF}"/>
    +</c:ext>
    +</c:extLst>
`officejs-chart_doughnut.golden.xlsx`, Sheet1, chart 1 (`xl/charts/chart1.xml`):
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    +<c16:uniqueId val="{00000001-255C-40ED-921C-0F085E61FBDB}"/>
    +</c:ext>
    +</c:extLst>
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    +<c16:uniqueId val="{00000003-255C-40ED-921C-0F085E61FBDB}"/>
    +</c:ext>
    +</c:extLst>
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    ... and 8 more lines
`officejs-chart_pie.golden.xlsx`, Sheet1, chart 1 (`xl/charts/chart1.xml`):
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    +<c16:uniqueId val="{00000001-8083-42A1-B433-E40BD515230A}"/>
    +</c:ext>
    +</c:extLst>
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    +<c16:uniqueId val="{00000003-8083-42A1-B433-E40BD515230A}"/>
    +</c:ext>
    +</c:extLst>
    +<c:extLst>
    +<c:ext uri="{C3380CC4-5D6E-409C-BE32-E72D297353CC}" xmlns:c16="http://schemas.microsoft.com/office/drawing/2014/chart">
    ... and 8 more lines
`waterfall.xlsx`, Sheet1, chart 1 (`xl/charts/chartEx1.xml`):
    +<cx:txPr>
    +<a:bodyPr vertOverflow="overflow" horzOverflow="overflow" wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" />
    +<a:lstStyle />
    +<a:p>
    +<a:pPr algn="ctr" rtl="0">
    +<a:defRPr sz="1200" b="0" i="0">
    +<a:solidFill>
    +<a:srgbClr val="000000" />
    +</a:solidFill>
    +<a:latin typeface="Calibri" panose="020F0502020204030204" pitchFamily="34" charset="0" />
    +<a:ea typeface="Calibri" panose="020F0502020204030204" pitchFamily="34" charset="0" />
    +<a:cs typeface="Calibri" panose="020F0502020204030204" pitchFamily="34" charset="0" />
    ... and 59 more lines
