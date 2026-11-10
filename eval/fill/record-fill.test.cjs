const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { test } = require("node:test");
const { runInNewContext } = require("node:vm");

const code = readFileSync(join(__dirname, "record-fill.js"), "utf8");
const recordFill = runInNewContext(`${code}\nrecordFill`);
const probe = {
  id: "mechanics-only", sha256: "a".repeat(64), sheet: "Fill", source: "A1:A2", target: "A1:A4",
  autoFillType: "FillDefault", direction: "down", length: 2, lengthUnit: "rows", merges: [],
};
const recordedAt = "2026-11-10T00:00:00Z";

function fixture(options = {}) {
  const state = { values: options.valuesBefore ?? [1, 3, "", ""], calls: [], calculations: [], reads: [], pending: [], phase: "before" };
  const propertyError = (name) => Object.assign(new Error(`PropertyNotLoaded: ${name}`), { code: "PropertyNotLoaded" });
  const loadable = (properties, automatic = []) => {
    const loaded = new Set();
    const object = { load(names) { state.pending.push(() => names.split(",").forEach((name) => loaded.add(name))); return object; } };
    state.pending.push(() => automatic.forEach((name) => loaded.add(name)));
    for (const [name, get] of Object.entries(properties)) {
      Object.defineProperty(object, name, { get() { if (!loaded.has(name)) throw propertyError(name); return get(); } });
    }
    return object;
  };
  const cell = (row, col) => {
    state.reads.push({ row, col, phase: state.phase });
    const range = loadable({
      address: () => options.wrongAddress ? "Fill!B1" : `Fill!A${row + 1}`,
      rowIndex: () => row, columnIndex: () => col, rowCount: () => 1, columnCount: () => 1,
      values: () => options.missingValue && row === 2 ? [] : [[state.values[row]]],
      valueTypes: () => [[options.valueTypes?.[row] ?? (typeof state.values[row] === "number" ? "Double" : "Empty")]],
      formulas: () => [[options.formulas?.[row] ?? state.values[row]]],
      numberFormat: () => [[options.numberFormats?.[row] ?? "General"]], text: () => [[String(state.values[row])]],
    });
    range.format = {
      font: loadable({ bold: () => options.missingFormat ? undefined : false, italic: () => false, color: () => "#000000" }),
      fill: loadable({ color: () => "" }),
    };
    if (!options.missingMergeApi) range.getMergedAreasOrNullObject = () => loadable({
      isNullObject: () => !options.merges,
      address: () => { if (!options.merges) throw new Error("Null object address read"); return "Fill!$A$1:$B$1"; },
    }, ["isNullObject"]);
    return range;
  };
  const source = {};
  if (!options.missingFillApi) source.autoFill = (target, type) => {
    state.calls.push({ target, type });
    state.pending.push(() => {
      state.phase = "after";
      if (options.unexpectedFailure) throw new TypeError("broken fixture");
      if (options.refusal) {
        if (options.partialMutation) state.values[2] = 99;
        throw Object.assign(new Error("recorded operation error"), { code: "InvalidArgument" });
      }
      state.values = [1, 3, 42, 43];
    });
  };
  const context = {
    application: { calculate(kind) { state.calculations.push(kind); } },
    workbook: { worksheets: { getItem(name) {
      assert.equal(name, "Fill");
      return { getRange(address) { assert.equal(address, probe.source); return source; }, getCell: cell };
    } } },
    async sync() {
      if (options.setupFailure && state.calls.length === 0) throw propertyError("setup");
      if (options.afterFailure && state.phase === "after") throw propertyError("after");
      const pending = state.pending.splice(0);
      for (const operation of pending) operation();
    },
  };
  return { context, state };
}

async function run(options, overrides = {}) {
  const { context, state } = fixture(options);
  const result = JSON.parse(JSON.stringify(await recordFill(context, { ...probe, ...overrides }, recordedAt)));
  return { result, state };
}

test("self-contained distributed recorder captures every cell before and after, with one explicit fill", async () => {
  assert.doesNotMatch(code, /\bimport\s|require\(/);
  const { result, state } = await run({});
  assert.equal(result.schema, "rustsheet.excel-fill-officejs/1");
  assert.deepEqual(result.before.cells.map((cell) => cell.value), [1, 3, "", ""]);
  assert.deepEqual(result.after.cells.map((cell) => cell.value), [1, 3, 42, 43]);
  for (const phase of ["before", "after"]) {
    assert.deepEqual(result[phase].cells.map((cell) => cell.a1), ["A1", "A2", "A3", "A4"]);
    assert.equal(state.reads.filter((read) => read.phase === phase).length, 4);
  }
  assert.deepEqual(state.calls, [{ target: "A1:A4", type: "FillDefault" }]);
  assert.deepEqual(state.calculations, ["Full", "Full"]);
  assert.deepEqual(result.outcome, { status: "completed" });
});

for (const autoFillType of ["FillCopy", "FillSeries", "FillDays", "FillWeekdays", "FillMonths", "FillYears"]) {
  test(`passes explicit ${autoFillType} unchanged`, async () => {
    const { state } = await run({}, { autoFillType });
    assert.equal(state.calls[0].type, autoFillType);
  });
}

test("records an operation error and any partial mutation, never retries", async () => {
  const { result, state } = await run({ refusal: true, partialMutation: true });
  assert.deepEqual(result.outcome, { status: "operation-error", code: "InvalidArgument", message: "recorded operation error" });
  assert.deepEqual(result.after.cells.map((cell) => cell.value), [1, 3, 99, ""]);
  assert.equal(state.calls.length, 1);
});

test("preserves date serials, error values, padded text and mixed formulas independently", async () => {
  const { result } = await run({
    valuesBefore: [45351, "Item007", "#REF!", 2],
    valueTypes: ["Double", "String", "Error", "Double"],
    formulas: [45351, "Item007", "=A0", "=A2+$A2+A$1+$A$1"],
    numberFormats: ["yyyy-mm-dd", "General", "General", "0.000"],
  });
  assert.deepEqual(result.before.cells.map((cell) => cell.value), [45351, "Item007", "#REF!", 2]);
  assert.deepEqual(result.before.cells.map((cell) => cell.formula), [45351, "Item007", "=A0", "=A2+$A2+A$1+$A$1"]);
  assert.deepEqual(result.before.cells.map((cell) => cell.numberFormat), ["yyyy-mm-dd", "General", "General", "0.000"]);
  assert.equal(result.before.cells[2].valueType, "Error");
});

test("deduplicates real merged addresses; null merged objects need no address read", async () => {
  assert.deepEqual((await run({ merges: true }, { merges: ["A1:B1"] })).result.before.merges, ["A1:B1"]);
  assert.deepEqual((await run({})).result.before.merges, []);
});

test("merge capability must round-trip the known seed before any fill", async () => {
  const { context, state } = fixture({});
  await assert.rejects(recordFill(context, { ...probe, merges: ["A1:B1"] }, recordedAt), /Merge observation does not match/);
  assert.equal(state.calls.length, 0);
});

for (const [option, message] of [
  ["missingValue", /Incomplete single-cell values/], ["missingFormat", /Missing cell format/],
  ["wrongAddress", /Wrong single-cell coverage/], ["missingMergeApi", /merge observation is unavailable/],
  ["missingFillApi", /autoFill is unavailable/], ["setupFailure", /PropertyNotLoaded/],
]) {
  test(`${option} is an infrastructure failure, not an observation`, async () => {
    const { context, state } = fixture({ [option]: true });
    await assert.rejects(recordFill(context, probe, recordedAt), message);
    assert.equal(state.calls.length, 0);
  });
}

test("unexpected fill failure and failed after-read cannot become successful records", async () => {
  await assert.rejects(run({ unexpectedFailure: true }), /broken fixture/);
  await assert.rejects(run({ afterFailure: true }), /PropertyNotLoaded/);
});

for (const changes of [
  { sha256: "unverified" }, { sheet: "Other" }, { target: "A1:B4" }, { target: "A1:A1" },
  { target: "A0:A4" }, { target: "A1:A1048577" }, { length: 3 }, { direction: "up" },
  { autoFillType: "FlashFill" }, { target: "A1:A2000", length: 1998 },
]) {
  test(`invalid request ${JSON.stringify(changes)} never queues a fill`, async () => {
    const { context, state } = fixture({});
    await assert.rejects(recordFill(context, { ...probe, ...changes }, recordedAt));
    assert.equal(state.calls.length, 0);
  });
}
