// Office.js (ExcelApi 1.8+), one workbook, one sheet per case (as Calipers' scripts). For each case: a trend
// row (the helper row T2:AE2 for the United States), a line chart of it by rows, its title and legend hidden, its
// axes shown or hidden, at a size in points. Then read the plot area and the value axis Excel computed, and the
// chart's picture. Nothing is set on the axes but `visible`, so minimum, maximum and majorUnit are Excel's automatic
// ones (Office.js returns the computed number when the value is automatic).
const TREND = [932, 948, 997, 1025, 1062, 1080, 1075, 1065, 1061, 1040, 1031, 1038];
// [id, width pt, height pt, axes visible]
const CASES = [
  ["S1", 48, 15, false], // one default cell (64 x 20 px): where the sparkline charts are anchored
  ["S2", 110, 18, false], // the size the script asked for (chart.width = 110, chart.height = 18)
  ["S3", 110, 36, false],
  ["S4", 110, 72, false],
  ["S5", 360, 216, false], // Excel's default chart size
  ["S6", 48, 15, true],
  ["S7", 110, 18, true],
  ["S8", 110, 36, true],
  ["S9", 110, 72, true],
  ["S10", 360, 216, true],
  // Rounded corners (c:roundedCorners): the picture shows Excel's radius at the default size.
  ["S11", 360, 216, true, { roundedCorners: true }],
];
async function runAll() {
  const out = {};
  for (const [id, w, h, axes, extra] of CASES) {
    await Excel.run(async (context) => {
      const sheet = context.workbook.worksheets.add(id);
      const data = sheet.getRange("T2:AE2");
      data.values = [TREND];
      const chart = sheet.charts.add(Excel.ChartType.line, data, Excel.ChartSeriesBy.rows);
      chart.title.visible = false;
      chart.legend.visible = false;
      chart.axes.categoryAxis.visible = axes;
      chart.axes.valueAxis.visible = axes;
      if (extra?.roundedCorners) chart.format.roundedCorners = true;
      chart.setPosition("S2");
      chart.width = w;
      chart.height = h;
      await context.sync();
      const plot = chart.plotArea;
      plot.load("left,top,width,height,insideLeft,insideTop,insideWidth,insideHeight,position");
      const v = chart.axes.valueAxis;
      v.load("minimum,maximum,majorUnit,visible");
      const g = v.majorGridlines;
      g.load("visible");
      chart.load("width,height");
      const image = chart.getImage(w * 4, h * 4);
      await context.sync();
      out[id] = {
        chart: { width: chart.width, height: chart.height },
        plotArea: plot.toJSON(),
        valueAxis: { minimum: v.minimum, maximum: v.maximum, majorUnit: v.majorUnit, visible: v.visible, gridlines: g.visible },
        imagePng: image.value,
      };
    });
  }
  return out; // save as small-chart-layout.answers.json
}
// runAll().then((r) => console.log(JSON.stringify(r)));
