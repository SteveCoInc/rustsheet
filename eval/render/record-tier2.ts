/**
 * The render-fidelity tier-2 recorder: one Office.js function for the Excel UI gym's `eval` operation. Given the
 * request context of a workbook the gym has open and that workbook's entry in probes.json, it returns the tier-2 dump
 * (rustsheet.excel-render-officejs/1): the tier-1 dump's shape and spellings (Office.js and Graph name things alike)
 * plus what Graph cannot read.
 *
 * Self-contained on purpose: no imports, nothing from the gym or from T1, so its TypeScript can be stripped to the
 * JavaScript the gym evaluates. Every property it reads is in the Office.js reference (ExcelApi requirement set noted
 * beside it); what the reference does not settle is marked unverified, and the first real run settles it. The gate
 * decodes the dump with its own shape, so a property that comes back other than expected is refused, never misread.
 *
 * Not readable through Office.js, so not here (tier 3, Excel desktop through COM): the text as fitted to the column
 * (`text` "will not depend on the cell width"), whether zero values are shown (Worksheet has no such property), and
 * desktop's own layout when the gym runs Excel for the web.
 */

/** The parts of the Office.js object model this function uses, structurally (the gym passes the real ones). */
interface Loadable {
  load(props: string): unknown;
}
interface OjsBorder extends Loadable {
  style: string;
  weight: string;
  color: string;
}
interface OjsRange extends Loadable {
  address: string;
  text: string[][];
  left: number;
  top: number;
  width: number;
  height: number;
  columnHidden: boolean;
  rowHidden: boolean;
  rowCount: number;
  columnCount: number;
  isNullObject: boolean;
  format: {
    horizontalAlignment: string;
    verticalAlignment: string;
    wrapText: boolean;
    indentLevel: number;
    shrinkToFit: boolean;
    textOrientation: number;
    font: { name: string; size: number; bold: boolean; italic: boolean; underline: string; strikethrough: boolean; color: string };
    fill: { color: string; pattern: string; patternColor: string };
    borders: { getItem(side: string): OjsBorder };
  };
  getMergedAreasOrNullObject(): OjsRange;
}
interface OjsChart {
  name: string;
  left: number;
  top: number;
  width: number;
  height: number;
  chartType: string;
}
interface OjsWorksheet extends Loadable {
  showGridlines: boolean;
  showHeadings: boolean;
  freezePanes: { getLocationOrNullObject(): OjsRange };
  getCell(row: number, column: number): OjsRange;
  charts: Loadable & { items: OjsChart[] };
}
export interface OjsContext {
  workbook: { worksheets: { getItem(name: string): OjsWorksheet } };
  sync(): Promise<unknown>;
}

/** The workbook's entry in probes.json (rustsheet.render-probes/1). */
export interface ProbeEntry {
  readonly id: string;
  readonly sha256: string;
  readonly sheets: ReadonlyArray<{ readonly name: string; readonly range: string; readonly charts: number }>;
}

const SIDES = ["EdgeLeft", "EdgeTop", "EdgeRight", "EdgeBottom", "DiagonalUp", "DiagonalDown"] as const;
const CELL_PROPS = [
  "address", "text", "left", "top", "width", "height", "columnHidden", "rowHidden", // left…height: ExcelApi 1.10
  "format/horizontalAlignment", "format/verticalAlignment", "format/wrapText",
  "format/indentLevel", "format/shrinkToFit", "format/textOrientation", // 1.9, 1.9, 1.7
  "format/font/name", "format/font/size", "format/font/bold", "format/font/italic", "format/font/underline", "format/font/color",
  "format/font/strikethrough", "format/fill/color", "format/fill/pattern", "format/fill/patternColor", // 1.9 for the three
].join(",");

/** "B4" from a 0-based row and column. */
function a1(row: number, col: number): string {
  let n = col + 1;
  let s = "";
  while (n > 0) {
    s = String.fromCharCode(65 + ((n - 1) % 26)) + s;
    n = Math.floor((n - 1) / 26);
  }
  return `${s}${row + 1}`;
}

/** "A1:S5" as 0-based bounds. */
function bounds(range: string): { r0: number; c0: number; r1: number; c1: number } {
  const cell = (ref: string): [number, number] => {
    const m = /^([A-Z]+)(\d+)$/.exec(ref);
    if (!m) throw new Error(`not an A1 cell: ${ref}`);
    let col = 0;
    for (const ch of m[1] as string) col = col * 26 + (ch.charCodeAt(0) - 64);
    return [Number(m[2]) - 1, col - 1];
  };
  const [from, to] = range.split(":");
  const [r0, c0] = cell(from ?? "");
  const [r1, c1] = cell(to ?? "");
  return { r0, c0, r1, c1 };
}

/** An address without its sheet and `$`s: "Sheet1!$B$2:$D$3" → "B2:D3". */
const bare = (address: string): string => (address.includes("!") ? address.slice(address.lastIndexOf("!") + 1) : address).replace(/\$/g, "");

/** The tier-2 dump of the workbook the context has open, for the probe `probe`. */
export async function recordRenderTier2(context: OjsContext, probe: ProbeEntry, recordedAt: string): Promise<unknown> {
  const sheets: unknown[] = [];
  for (const s of probe.sheets) {
    const ws = context.workbook.worksheets.getItem(s.name);
    ws.load("showGridlines,showHeadings"); // ExcelApi 1.8
    const frozen = ws.freezePanes.getLocationOrNullObject(); // ExcelApi 1.7
    frozen.load("address,rowCount,columnCount");
    ws.charts.load("items/name,items/left,items/top,items/width,items/height,items/chartType"); // chartType: 1.7
    const b = bounds(s.range);
    const cells: Array<{ row: number; col: number; range: OjsRange; borders: Array<[string, OjsBorder]>; merged: OjsRange }> = [];
    for (let row = b.r0; row <= b.r1; row++) {
      for (let col = b.c0; col <= b.c1; col++) {
        const range = ws.getCell(row, col);
        range.load(CELL_PROPS);
        const borders = SIDES.map((side): [string, OjsBorder] => {
          const border = range.format.borders.getItem(side);
          border.load("style,weight,color");
          return [side, border];
        });
        const merged = range.getMergedAreasOrNullObject(); // the merge the cell is in, or a null object
        merged.load("address");
        cells.push({ row, col, range, borders, merged });
      }
    }
    await context.sync();
    const merges = new Set<string>();
    for (const c of cells) if (!c.merged.isNullObject) merges.add(bare(c.merged.address));
    // The cell's own line sizes are its Width and Height (points, documented); format.columnWidth states no unit.
    const first = (pick: (c: (typeof cells)[number]) => boolean) => cells.find(pick) as (typeof cells)[number];
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
      cells: cells.map(({ row, col, range: r, borders }) => ({
        a1: a1(row, col),
        text: r.text[0]?.[0] ?? "",
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
        borders: Object.fromEntries(borders.map(([side, x]) => [side, { style: x.style, weight: x.weight, color: x.color }])),
      })),
      charts: ws.charts.items.map((c) => ({ name: c.name, left: c.left, top: c.top, width: c.width, height: c.height, type: c.chartType })),
    });
  }
  return { schema: "rustsheet.excel-render-officejs/1", probe: probe.id, sha256: probe.sha256, source: { api: "office-js", recordedAt }, sheets };
}

function pickFont(f: OjsRange["format"]["font"]): Record<string, unknown> {
  return { name: f.name, size: f.size, bold: f.bold, italic: f.italic, underline: f.underline, strikethrough: f.strikethrough, color: f.color };
}
