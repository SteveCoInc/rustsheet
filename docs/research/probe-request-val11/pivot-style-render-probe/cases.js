// Office.js, run inside pivot-style-render.xlsx opened in Excel for the web (the workbook's pivots are
// refreshOnLoad, so Excel lays them out on open). Nothing is written: for each pivot the script reads where
// Excel put it, the format Excel displays on every cell of it, the format each cell holds itself, and a picture.
//
// API sets (learn.microsoft.com, Excel.Range and Excel.PivotLayout, read 2026-11-12):
//   PivotLayout.getRange, getFilterAxisRange, getRowLabelRange, getColumnLabelRange, getDataBodyRange: ExcelApi 1.8
//   Range.getCellProperties: ExcelApi 1.9        Range.getImage: ExcelApi 1.7
//   Range.getMergedAreasOrNullObject: ExcelApi 1.13 (the merged ranges inside the pivot's range, or none)
//   Range.getDisplayedCellProperties: ExcelApi 1.19 ("properties that are displayed from indirect sources, such
//   as conditional formatting or styles"). Where the host lacks 1.19 the answer says so and holds the cells' own
//   formats only: that is still an answer (the probe then has no oracle for what a style paints).
const SHEETS = ["L16", "L16B", "M9", "M9B", "D2", "D2B", "L16N", "L16S", "L16X", "L16C", "AC", "AO", "AT", "AM", "AI"];
const LOAD = {
  address: true,
  format: {
    fill: { color: true, pattern: true },
    font: { bold: true, italic: true, color: true, underline: true, strikethrough: true },
    borders: { color: true, style: true, weight: true },
    horizontalAlignment: true,
    indentLevel: true,
  },
};
async function runAll() {
  const displayed = Office.context.requirements.isSetSupported("ExcelApi", "1.19");
  const out = { host: Office.context.diagnostics, displayedCellProperties: displayed, pivots: {} };
  for (const name of SHEETS) {
    try {
      await readPivot(name, displayed, out);
    } catch (error) {
      // One pivot's failure is recorded and the others are still read.
      out.pivots[name] = { error: String(error), debug: error && error.debugInfo ? error.debugInfo : null };
    }
  }
  return out; // save as pivot-style-render.answers.json
}
async function readPivot(name, displayed, out) {
  {
    await Excel.run(async (context) => {
      const sheet = context.workbook.worksheets.getItem(name);
      const pivot = sheet.pivotTables.getItem("P" + name);
      const layout = pivot.layout;
      const parts = {
        range: layout.getRange(),
        rowLabels: layout.getRowLabelRange(),
        columnLabels: layout.getColumnLabelRange(),
        dataBody: layout.getDataBodyRange(),
      };
      for (const part of Object.values(parts)) part.load("address,rowCount,columnCount");
      // The filter area exists only with a report filter (the three A sheets have none).
      const filters = name.startsWith("A") ? null : layout.getFilterAxisRange();
      if (filters) filters.load("address");
      await context.sync();
      const range = parts.range;
      range.load("text,values");
      const own = range.getCellProperties(LOAD);
      const shown = displayed ? range.getDisplayedCellProperties(LOAD) : null;
      const image = range.getImage();
      // Where the host lacks ExcelApi 1.13 the answer's `merged` is "unsupported", and the rest is still read.
      const canMerge = Office.context.requirements.isSetSupported("ExcelApi", "1.13");
      const merged = canMerge ? range.getMergedAreasOrNullObject() : null;
      if (merged) merged.load("address");
      let filterOwn = null, filterShown = null, filterText = null;
      if (filters) {
        filters.load("text");
        filterOwn = filters.getCellProperties(LOAD);
        filterShown = displayed ? filters.getDisplayedCellProperties(LOAD) : null;
        filterText = filters;
      }
      await context.sync();
      out.pivots[name] = {
        address: range.address,
        rowLabels: parts.rowLabels.address,
        columnLabels: parts.columnLabels.address,
        dataBody: parts.dataBody.address,
        text: range.text,
        values: range.values,
        own: own.value,
        displayed: shown ? shown.value : null,
        filters: filters
          ? { address: filters.address, text: filterText.text, own: filterOwn.value, displayed: filterShown ? filterShown.value : null }
          : null,
        merged: !merged ? "unsupported" : merged.isNullObject ? null : merged.address,
        imagePng: image.value,
      };
    });
  }
}
// runAll().then((r) => console.log(JSON.stringify(r)));
