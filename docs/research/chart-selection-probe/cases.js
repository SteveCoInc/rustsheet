// Office.js, one case per run in a fresh workbook (as Calipers' scripts): the selection rule cases of README.md.
// Dates are serials shown as mmm-yy (45292 is 2024-01-01).
const months = Array.from({ length: 12 }, (_, i) => 45292 + 31 * i);
const cases = {
  P1: { range: "A1:D13", values: [["Month", "Revenue", "Cost", "Margin"], ...months.map((d, i) => [d, 100 + i, 60 + i, 0.4])], dates: "A2:A13", datesShape: [12, 1] },
  P2: { range: "A1:A5", values: [["Cost"], [4], [5], [7], [6]] },
  P3: { range: "A1:D1", values: [["Cost", 4, 5, 7]] },
  P4: { range: "A1:B6", values: [["Year", "Sales"], [2014, 10], [2015, 12], [2016, 9], [2017, 14], [2018, 15]] },
  P5: { range: "A1:C4", values: [["", 2023, 2024], ["North", 5, 6], ["South", 7, 8], ["East", 1, 2]] },
  P6: { range: "A1:C4", values: [["Region", 45292, 45323], ["North", 5, 6], ["South", 7, 8], ["East", 1, 2]], dates: "B1:C1", datesShape: [1, 2] },
  P7: { range: "A1:C4", values: [["Region", 10, 20], ["North", 5, 6], ["South", 7, 8], ["East", 1, 2]] },
};
async function run(id) {
  const c = cases[id];
  await Excel.run(async (context) => {
    const sheet = context.workbook.worksheets.getActiveWorksheet();
    sheet.name = id;
    const range = sheet.getRange(c.range);
    range.values = c.values;
    if (c.dates) {
      const [rows, cols] = c.datesShape;
      sheet.getRange(c.dates).numberFormat = Array.from({ length: rows }, () => Array(cols).fill("mmm-yy"));
    }
    sheet.charts.add(Excel.ChartType.columnClustered, range, Excel.ChartSeriesBy.auto);
    await context.sync();
  });
}
// run("P1"); run("P2"); ... one per fresh workbook, or one sheet each in one workbook.
