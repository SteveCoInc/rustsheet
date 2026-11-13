// Office.js, run once inside each of the three workbooks opened in Excel for the web. It makes one pivot on the
// empty sheet New (rows Region, columns Product, Sum of Sales, from Data!A1:C7) and reads what Excel shows for it.
// The workbooks differ only in the default pivot style their styles part names, so the answers say which style
// Excel for the web gives a new pivot. Office.js has no member that reads a pivot's style name (learn.microsoft.com,
// Excel.PivotLayout, read 2026-11-12), so the style is told from the displayed format of the pivot's cells and,
// for certain, from the downloaded workbook (README.md, step 3).
//
// API sets: PivotTableCollection.add, the hierarchy collections and PivotLayout.getRange: ExcelApi 1.8.
//   Range.getCellProperties: ExcelApi 1.9. Range.getImage: ExcelApi 1.7.
//   Range.getDisplayedCellProperties: ExcelApi 1.19; where the host lacks it the answer says so.
const LOAD = {
  address: true,
  format: {
    fill: { color: true, pattern: true },
    font: { bold: true, color: true },
    borders: { color: true, style: true, weight: true },
  },
};
async function runAll() {
  const displayed = Office.context.requirements.isSetSupported("ExcelApi", "1.19");
  const out = { host: Office.context.diagnostics, displayedCellProperties: displayed };
  try {
    await Excel.run(async (context) => {
      const data = context.workbook.worksheets.getItem("Data").getRange("A1:C7");
      const sheet = context.workbook.worksheets.getItem("New");
      const pivot = sheet.pivotTables.add("Made", data, sheet.getRange("A3"));
      pivot.rowHierarchies.add(pivot.hierarchies.getItem("Region"));
      pivot.columnHierarchies.add(pivot.hierarchies.getItem("Product"));
      pivot.dataHierarchies.add(pivot.hierarchies.getItem("Sales"));
      await context.sync();
      const range = pivot.layout.getRange();
      range.load("address,text,values");
      const own = range.getCellProperties(LOAD);
      const shown = displayed ? range.getDisplayedCellProperties(LOAD) : null;
      const image = range.getImage();
      await context.sync();
      out.pivot = {
        address: range.address,
        text: range.text,
        values: range.values,
        own: own.value,
        displayed: shown ? shown.value : null,
        image: image.value, // base64 PNG
      };
    });
  } catch (error) {
    out.error = String(error);
    out.debug = error && error.debugInfo ? error.debugInfo : null;
  }
  return out; // save as pivot-default-style-<m9|none|own>.answers.json
}
