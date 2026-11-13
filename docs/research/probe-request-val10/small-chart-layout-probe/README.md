# Probe request: Excel's plot area and value-axis scale for small line charts with hidden axes

*From poc-charts-cc, 2026-11-11. For the Excel oracle owner, routed by the coordinator. Wanted soon: it decides
whether D-CHART-50's reading (the "monthly trend" fix) is Excel's.*

## Why

A script added 12 cell-sized line charts as sparklines: `sheet.charts.add(line, T{r}:AE{r},
rows)`, with the title, legend and both axes `visible = false`, at `chart.setPosition("S{r}")` (one 48 x 15 pt cell),
asking for 110 x 18 pt. Our layout drew each one as a flat line. It used fixed margins measured on default-size
charts, which left no plot at 20 px high, and the major-unit rule for labelled axes, which gave a coarse unit on a tiny
plot. poc/charts now (D-CHART-50):
- keeps the measured margins, but never lets the two margins of a side take more than a quarter of the chart;
- leaves out the half label over the plot when the value axis shows no labels;
- for an axis that shows no labels, applies only the 10-interval cap to the unit, not the label spacing.

So for the trend 932..1080 at 48 x 15 pt we now draw an 850..1100 axis by 50, a 48 x 15 px plot, and the line
spanning 59% of the plot (test/trend-chart.test.ts on that document). **None of this is verified.** Excel may keep its margins at any size, or space the unit by
the hidden axis' label size. If so, Excel draws the same flat line, and the fix belongs in the script, not the
renderer.

## How

Run `cases.js` (`runAll()`) in Excel for the web or desktop, through Office.js. Save the JSON it returns as
`small-chart-layout.answers.json` next to this file, and the workbook as `small-chart-layout.golden.xlsx`. The
workbook gives us each chart part, and the JSON gives Excel's computed layout and scale.

## The cases and our current reading

All cases use the same data: one row, T2:AE2 = 932 948 997 1025 1062 1080 1075 1065 1061 1040 1031 1038. The chart
is a line chart of it by rows, with its title and legend hidden.

| Case | Size (pt) | Axes | Our plot, inside (px at 96 DPI: x, y, w, h) | Our value axis |
|---|---|---|---|---|
| S1 | 48 x 15 | hidden | 6.08, 2.50, 48.00, 15.00 | 850-1100 by 50 |
| S2 | 110 x 18 | hidden | 9.10, 3.00, 122.72, 18.00 | 850-1100 by 50 |
| S3 | 110 x 36 | hidden | 9.10, 6.00, 122.72, 36.00 | 850-1100 by 50 |
| S4 | 110 x 72 | hidden | 9.10, 9.10, 122.72, 77.80 | 850-1100 by 50 |
| S5 | 360 x 216 | hidden | 9.10, 9.10, 456.05, 269.80 | 850-1100 by 50 |
| S6 | 48 x 15 | shown | 43.11, 9.82, 10.97, 0.00 (the labels take it all) | 850-1100 by 50 |
| S7 | 110 x 18 | shown | 46.13, 10.32, 85.69, 0.00 | 850-1100 by 50 |
| S8 | 110 x 36 | shown | 46.13, 13.32, 85.69, 1.27 | 0-2000 by 2000 |
| S9 | 110 x 72 | shown | 46.13, 16.42, 85.69, 43.07 | 800-1200 by 200 |
| S10 | 360 x 216 | shown | 46.13, 16.42, 419.02, 239.38 | 850-1100 by 50 |
| S11 | 360 x 216 | shown, `format.roundedCorners = true` | as S10 | as S10; its picture shows Excel's corner radius (ours: 10 px) |

(Measured on poc/charts 431de6855 with `rustsheet_charts::layout`; the chart is Insert's line chart of the row, title
and legend removed. Points to px: x 4/3.)

Note: the plot is what is left after the margins. Each margin is Excel's measured one (left 9.1, right 14.85, top and
bottom 9.1 px) or, where the two margins of a side would take more than a quarter of the chart, both margins scaled
down to a quarter.

For each case we read `plotArea` (`insideLeft/Top/Width/Height`, in points) and `valueAxis` (`minimum`, `maximum`,
`majorUnit`, the automatic values Excel computed). The picture is a cross-check.
