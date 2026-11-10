async function recordFill(context, probe, recordedAt) {
  const fillTypes = ["FillDefault", "FillCopy", "FillSeries", "FillDays", "FillWeekdays", "FillMonths", "FillYears"];
  if (!probe.id || probe.sheet !== "Fill" || !/^[a-f0-9]{64}$/.test(probe.sha256) || !fillTypes.includes(probe.autoFillType)) {
    throw new Error("Invalid pinned fill probe");
  }
  if (!/^\d{4}-\d{2}-\d{2}T/.test(recordedAt) || !Number.isFinite(Date.parse(recordedAt))) {
    throw new Error("recordedAt must be an ISO timestamp supplied by the recorder owner");
  }
  const cellPosition = (address) => {
    const match = /^([A-Z]{1,3})([1-9][0-9]*)$/.exec(address);
    if (!match) throw new Error(`Not a plain A1 cell: ${address}`);
    let column = 0;
    for (const letter of match[1]) column = column * 26 + letter.charCodeAt(0) - 64;
    const row = Number(match[2]);
    if (row > 1048576 || column > 16384) throw new Error(`Cell is off grid: ${address}`);
    return [row - 1, column - 1];
  };
  const rectangle = (address) => {
    const parts = address.split(":");
    if (parts.length > 2) throw new Error(`Not a contiguous range: ${address}`);
    const [r0, c0] = cellPosition(parts[0]);
    const [r1, c1] = cellPosition(parts[1] ?? parts[0]);
    if (r0 > r1 || c0 > c1) throw new Error(`Reversed range: ${address}`);
    return { r0, c0, r1, c1 };
  };
  const sourceBounds = rectangle(probe.source);
  const targetBounds = rectangle(probe.target);
  const edges = ["r0", "c0", "r1", "c1"];
  const changed = edges.filter((edge2) => sourceBounds[edge2] !== targetBounds[edge2]);
  if (changed.length !== 1 || targetBounds.r0 > sourceBounds.r0 || targetBounds.c0 > sourceBounds.c0 || targetBounds.r1 < sourceBounds.r1 || targetBounds.c1 < sourceBounds.c1) throw new Error("Target must extend one edge of source");
  const edge = changed[0];
  const direction = { r0: "up", r1: "down", c0: "left", c1: "right" }[edge];
  const length = Math.abs(targetBounds[edge] - sourceBounds[edge]);
  const lengthUnit = edge.startsWith("r") ? "rows" : "columns";
  if (probe.direction !== direction || probe.length !== length || probe.lengthUnit !== lengthUnit) throw new Error("Fill metadata disagrees with ranges");
  const count = (targetBounds.r1 - targetBounds.r0 + 1) * (targetBounds.c1 - targetBounds.c0 + 1);
  if (count > 1024) throw new Error("Probe is too large for complete per-cell recording");
  const sheet = context.workbook.worksheets.getItem(probe.sheet);
  const source = sheet.getRange(probe.source);
  if (typeof source.autoFill !== "function") throw new Error("ExcelApi 1.9 Range.autoFill is unavailable");
  const bare = (address) => address.slice(address.lastIndexOf("!") + 1).replace(/\$/g, "");
  const addressOf = (row, col) => {
    let letters = "";
    for (let remaining = col + 1; remaining > 0; remaining = Math.floor((remaining - 1) / 26)) {
      letters = String.fromCharCode(65 + (remaining - 1) % 26) + letters;
    }
    return `${letters}${row + 1}`;
  };
  const single = (matrix, label) => {
    if (!Array.isArray(matrix) || matrix.length !== 1 || !Array.isArray(matrix[0]) || matrix[0].length !== 1 || matrix[0][0] === void 0) {
      throw new Error(`Incomplete single-cell ${label}`);
    }
    return matrix[0][0];
  };
  const scalar = (value, label) => {
    if (value === null || typeof value === "string" || typeof value === "boolean" || typeof value === "number" && Number.isFinite(value)) return value;
    throw new Error(`Invalid cell scalar ${label}`);
  };
  const snapshot = async () => {
    const pending = [];
    for (let row = targetBounds.r0; row <= targetBounds.r1; row++) {
      for (let col = targetBounds.c0; col <= targetBounds.c1; col++) {
        const range = sheet.getCell(row, col);
        range.load("address,rowIndex,columnIndex,rowCount,columnCount,values,valueTypes,formulas,numberFormat,text");
        range.format.font.load("bold,italic,color");
        range.format.fill.load("color");
        if (typeof range.getMergedAreasOrNullObject !== "function") throw new Error("ExcelApi 1.13 merge observation is unavailable");
        const merged = range.getMergedAreasOrNullObject();
        merged.load("address");
        pending.push({ row, col, range, merged });
      }
    }
    await context.sync();
    const merges = /* @__PURE__ */ new Set();
    const cells = pending.map(({ row, col, range, merged }) => {
      const address = addressOf(row, col);
      if (bare(range.address) !== address || range.rowIndex !== row || range.columnIndex !== col || range.rowCount !== 1 || range.columnCount !== 1) {
        throw new Error(`Wrong single-cell coverage at ${address}`);
      }
      if (typeof merged.isNullObject !== "boolean") throw new Error(`Missing merge observation at ${address}`);
      if (!merged.isNullObject) {
        if (typeof merged.address !== "string" || !merged.address) throw new Error(`Missing merged address at ${address}`);
        merges.add(bare(merged.address));
      }
      const valueType = single(range.valueTypes, "valueTypes");
      const numberFormat = single(range.numberFormat, "numberFormat");
      const text = single(range.text, "text");
      if ([valueType, numberFormat, text].some((value) => typeof value !== "string")) throw new Error(`Missing cell metadata at ${address}`);
      const { bold, italic, color } = range.format.font;
      const fillColor = range.format.fill.color;
      if ([bold, italic].some((value) => value !== null && typeof value !== "boolean") || [color, fillColor].some((value) => value !== null && typeof value !== "string")) throw new Error(`Missing cell format at ${address}`);
      return {
        a1: address,
        row,
        col,
        value: scalar(single(range.values, "values"), "value"),
        valueType,
        formula: scalar(single(range.formulas, "formulas"), "formula"),
        numberFormat,
        text,
        font: { bold, italic, color },
        fill: { color: fillColor }
      };
    });
    if (cells.length !== count || new Set(cells.map((cell) => cell.a1)).size !== count) throw new Error("Incomplete target coverage");
    return { cells, merges: [...merges].sort() };
  };
  context.application.calculate("Full");
  await context.sync();
  const before = await snapshot();
  if (!Array.isArray(probe.merges) || probe.merges.some((merge) => typeof merge !== "string") || JSON.stringify(before.merges) !== JSON.stringify([...probe.merges].sort())) {
    throw new Error("Merge observation does not match the pinned seed; no fill was attempted");
  }
  let outcome;
  try {
    source.autoFill(probe.target, probe.autoFillType);
    await context.sync();
    outcome = { status: "completed" };
  } catch (error) {
    if (!error || typeof error !== "object" || !("code" in error) || typeof error.code !== "string" || !("message" in error) || typeof error.message !== "string") throw error;
    outcome = { status: "operation-error", code: error.code, message: error.message };
  }
  context.application.calculate("Full");
  await context.sync();
  const after = await snapshot();
  return {
    schema: "rustsheet.excel-fill-officejs/1",
    probe: probe.id,
    sha256: probe.sha256,
    source: { api: "office-js", recordedAt },
    sheet: probe.sheet,
    operation: { source: probe.source, target: probe.target, autoFillType: probe.autoFillType, direction, length, lengthUnit },
    outcome,
    before,
    after
  };
}
