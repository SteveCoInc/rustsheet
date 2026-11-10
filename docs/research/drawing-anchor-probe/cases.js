// Office.js, one case per run in a fresh workbook (or one sheet each): the drawing-anchor cases of README.md.
// The chart's corners are mid-cell: top-left in the middle of C3, bottom-right in the middle of H12 (default grid).
const ops = {
  R1: (s) => { s.getRange("3:3").format.rowHeight = 30; },
  R2: (s) => { s.getRange("12:12").format.rowHeight = 30; },
  R3: (s) => { s.getRange("6:6").format.rowHeight = 30; },
  R4: (s) => { s.getRange("3:3").format.rowHeight = 5; },
  C1: (s) => { s.getRange("C:C").format.columnWidth = 20 * 7.5; }, // ~20 characters of Calibri 11, in points
  H1: (s) => { s.getRange("3:3").rowHidden = true; },
  H2: (s) => { s.getRange("12:12").rowHidden = true; },
  H3: (s) => { s.getRange("6:6").rowHidden = true; },
  H4: (s) => { s.getRange("C:C").columnHidden = true; },
  D1: (s) => { s.getRange("3:3").delete(Excel.DeleteShiftDirection.up); },
  D2: (s) => { s.getRange("12:12").delete(Excel.DeleteShiftDirection.up); },
  D3: (s) => { s.getRange("3:12").delete(Excel.DeleteShiftDirection.up); },
  D4: (s) => { s.getRange("C:C").delete(Excel.DeleteShiftDirection.left); },
};
async function run(id) {
  await Excel.run(async (context) => {
    const sheet = context.workbook.worksheets.getActiveWorksheet();
    sheet.name = id;
    sheet.getRange("A1:B4").values = [["Name", "Value"], ["a", 10], ["b", 20], ["c", 15]];
    const c3 = sheet.getRange("C3");
    const h12 = sheet.getRange("H12");
    c3.load("left,top,width,height");
    h12.load("left,top,width,height");
    await context.sync();
    const chart = sheet.charts.add(Excel.ChartType.columnClustered, sheet.getRange("A1:B4"), Excel.ChartSeriesBy.auto);
    chart.left = c3.left + c3.width / 2;
    chart.top = c3.top + c3.height / 2;
    chart.width = h12.left + h12.width / 2 - chart.left;
    chart.height = h12.top + h12.height / 2 - chart.top;
    chart.load("left,top,width,height");
    await context.sync();
    const before = { left: chart.left, top: chart.top, width: chart.width, height: chart.height };
    ops[id](sheet);
    await context.sync();
    const charts = sheet.charts;
    charts.load("items/left,items/top,items/width,items/height");
    await context.sync();
    const after = charts.items.map((c) => ({ left: c.left, top: c.top, width: c.width, height: c.height }));
    console.log(JSON.stringify({ id, before, after }));
  });
}
// run("R1"); ... then save the workbook; xdr:from / xdr:to come from its drawing part.
