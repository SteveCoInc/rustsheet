const SIDES = ["EdgeLeft", "EdgeTop", "EdgeRight", "EdgeBottom", "DiagonalUp", "DiagonalDown"];
const CELL_PROPS = [
  "address",
  "text",
  "left",
  "top",
  "width",
  "height",
  "columnHidden",
  "rowHidden",
  // left…height: ExcelApi 1.10
  "format/horizontalAlignment",
  "format/verticalAlignment",
  "format/wrapText",
  "format/indentLevel",
  "format/shrinkToFit",
  "format/textOrientation",
  // 1.9, 1.9, 1.7
  "format/font/name",
  "format/font/size",
  "format/font/bold",
  "format/font/italic",
  "format/font/underline",
  "format/font/color",
  "format/font/strikethrough",
  "format/fill/color",
  "format/fill/pattern",
  "format/fill/patternColor"
  // 1.9 for the three
].join(",");
function a1(row, col) {
  let n = col + 1;
  let s = "";
  while (n > 0) {
    s = String.fromCharCode(65 + (n - 1) % 26) + s;
    n = Math.floor((n - 1) / 26);
  }
  return `${s}${row + 1}`;
}
function bounds(range) {
  const cell = (ref) => {
    const m = /^([A-Z]+)(\d+)$/.exec(ref);
    if (!m) throw new Error(`not an A1 cell: ${ref}`);
    let col = 0;
    for (const ch of m[1]) col = col * 26 + (ch.charCodeAt(0) - 64);
    return [Number(m[2]) - 1, col - 1];
  };
  const [from, to] = range.split(":");
  const [r0, c0] = cell(from != null ? from : "");
  const [r1, c1] = cell(to != null ? to : "");
  return { r0, c0, r1, c1 };
}
const bare = (address) => (address.includes("!") ? address.slice(address.lastIndexOf("!") + 1) : address).replace(/\$/g, "");
async function recordRenderTier2(context, probe, recordedAt) {
  const sheets = [];
  for (const s of probe.sheets) {
    const ws = context.workbook.worksheets.getItem(s.name);
    ws.load("showGridlines,showHeadings");
    const frozen = ws.freezePanes.getLocationOrNullObject();
    frozen.load("address,rowCount,columnCount");
    ws.charts.load("items/name,items/left,items/top,items/width,items/height,items/chartType");
    const b = bounds(s.range);
    const cells = [];
    for (let row = b.r0; row <= b.r1; row++) {
      for (let col = b.c0; col <= b.c1; col++) {
        const range = ws.getCell(row, col);
        range.load(CELL_PROPS);
        const borders = SIDES.map((side) => {
          const border = range.format.borders.getItem(side);
          border.load("style,weight,color");
          return [side, border];
        });
        const merged = range.getMergedAreasOrNullObject();
        merged.load("address");
        cells.push({ row, col, range, borders, merged });
      }
    }
    await context.sync();
    const merges = /* @__PURE__ */ new Set();
    for (const c of cells) if (!c.merged.isNullObject) merges.add(bare(c.merged.address));
    const first = (pick) => cells.find(pick);
    const columns = [];
    for (let col = b.c0; col <= b.c1; col++) {
      const c = first((x) => x.col === col);
      columns.push({ index: col + 1, widthPt: c.range.width, hidden: c.range.columnHidden });
    }
    const rows = [];
    for (let row = b.r0; row <= b.r1; row++) {
      const c = first((x) => x.row === row);
      rows.push({ index: row + 1, heightPt: c.range.height, hidden: c.range.rowHidden });
    }
    sheets.push({
      name: s.name,
      // The frozen range's extent as Office.js reports it; which extent a rows-only freeze reports is unverified, so
      // the gate, not this function, reads frozen rows and columns out of it.
      window: { showGridlines: ws.showGridlines, showHeadings: ws.showHeadings, freeze: frozen.isNullObject ? null : { address: bare(frozen.address), rowCount: frozen.rowCount, columnCount: frozen.columnCount } },
      columns,
      rows,
      merges: [...merges].sort(),
      cells: cells.map(({ row, col, range: r, borders }) => {
        var _a, _b;
        return {
          a1: a1(row, col),
          text: (_b = (_a = r.text[0]) == null ? void 0 : _a[0]) != null ? _b : "",
          left: r.left,
          top: r.top,
          width: r.width,
          height: r.height,
          hAlign: r.format.horizontalAlignment,
          vAlign: r.format.verticalAlignment,
          wrap: r.format.wrapText,
          indent: r.format.indentLevel,
          shrink: r.format.shrinkToFit,
          orientation: r.format.textOrientation,
          font: { ...pickFont(r.format.font) },
          fill: { color: r.format.fill.color, pattern: r.format.fill.pattern, patternColor: r.format.fill.patternColor },
          borders: Object.fromEntries(borders.map(([side, x]) => [side, { style: x.style, weight: x.weight, color: x.color }]))
        };
      }),
      charts: ws.charts.items.map((c) => ({ name: c.name, left: c.left, top: c.top, width: c.width, height: c.height, type: c.chartType }))
    });
  }
  return { schema: "rustsheet.excel-render-officejs/1", probe: probe.id, sha256: probe.sha256, source: { api: "office-js", recordedAt }, sheets };
}
function pickFont(f) {
  return { name: f.name, size: f.size, bold: f.bold, italic: f.italic, underline: f.underline, strikethrough: f.strikethrough, color: f.color };
}
export {
  recordRenderTier2
};
