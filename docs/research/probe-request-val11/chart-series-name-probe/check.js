// Checks cases.js without Excel: node check.js
//
// It runs the three cases against a small fake of Office.js: a sheet of cells, names of either scope, and one
// chart of one series whose sources are kept as text. The fake resolves a source its own way, so it says
// nothing about Excel: what Excel does is what the request asks. It is strict where a real host is: a property
// is readable only after a load that names it and a sync, a result and `isNullObject` only after a sync, and an
// item that is not there fails the sync that asks for it. Checked here:
// - every run logs exactly one line of JSON, whatever happens: an unknown case, a workbook that is not the
//   probe's, a call Excel refuses, any one sync that fails, and `Excel.run` itself rejecting;
// - a workbook with no Sheet1, no chart, two charts or two series is neither read further nor changed, and the
//   line says what was found;
// - READ changes one cell, the marker in E1, and nothing else, and writes it after everything is read;
// - CHANGE writes B2, then gives `Sales` its new formula, then deletes it, in the scope the name has, and
//   reads the names and the series after each; with no name `Sales` it says so and changes B2 only;
// - SET runs in a fresh control only; it sets the series to C2:C5 before each attempt, makes its attempts in
//   the order README.md lists them, and one that is refused does not stop the next;
// - in Script Lab there is one button a case, and a click logs the case's line and shows it;
// - README.md names the cases, the spellings and the files the script and the generator hold.
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vm = require("vm");
const { fakePage, checkButtons } = require("../script_lab_page");

const source = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");
const readme = fs.readFileSync(path.join(__dirname, "README.md"), "utf8");

const RANGES = { Label: "Sheet1!$B$1", Months: "Sheet1!$A$2:$A$5", Sales: "Sheet1!$B$2:$B$5" };
const CELLS = [
  ["Month", "Value", "Other"],
  ["a", 10, 1],
  ["b", 20, 2],
  ["c", 15, 3],
  ["d", 25, 4],
];
// The six generated workbooks, as generate.py's FILES has them: the names' scope, a formula's spelling of a
// name, and the names the series spells so.
const ALL = Object.keys(RANGES);
const FILES = {
  "series-control.xlsx": ["Workbook", "", []],
  "series-workbook-name-values.xlsx": ["Workbook", "[0]!", ["Sales"]],
  "series-workbook-name.xlsx": ["Workbook", "[0]!", ALL],
  "series-sheet-name.xlsx": ["Worksheet", "Sheet1!", ALL],
  "series-bare-name.xlsx": ["Workbook", "", ALL],
  "series-prefixed-workbook-name.xlsx": ["Workbook", "Sheet1!", ALL],
};

function refused(code) {
  return Object.assign(new Error(code), { code, debugInfo: { errorLocation: "fake" } });
}

// `fail`: the number (from 1) of a sync that rejects, dropping what it queued. `strings`: whether a series
// takes a string. `sheet`, `charts`, `series`: what the workbook holds. `sales`: whether `Sales` is defined.
// `runRejects`: `Excel.run` rejects after the batch, as a real one does when its own last sync fails.
function makeHost(file, options = {}) {
  const { fail = 0, strings = true, sheet = true, charts = 1, series = 1, sales = true, runRejects = false } = options;
  const { b2, gate } = options; // what B2 holds, where it is not the generator's 10; what Excel.run waits for
  const [scope, prefix, named] = FILES[file];
  const spell = (name) => (named.includes(name) ? prefix + name : RANGES[name]);
  const state = {
    cells: CELLS.map((row) => [...row]),
    names: Object.keys(RANGES)
      .filter((name) => sales || name !== "Sales")
      .map((name) => ({ name, scope, formula: "=" + RANGES[name] })),
    seriesName: "stale",
    sources: { Categories: spell("Months"), Values: spell("Sales") },
  };
  if (b2 !== undefined) state.cells[1][1] = b2;
  const host = { state, syncs: 0, changes: [], calls: [], lines: [] };
  let queued = [];
  let batch = 0; // the number of syncs that have run their queue
  const queue = (work) => queued.push(work);
  // Properties that are readable once a load named them and a sync has run since.
  const loadable = (object, properties) => {
    const loaded = new Map();
    object.load = (names) => {
      // "items/name" loads `name` of each item: it makes `items` readable.
      for (const name of String(names).split(",")) loaded.set(name.split("/")[0], batch);
      queue(() => {});
    };
    for (const [name, read] of Object.entries(properties)) {
      Object.defineProperty(object, name, {
        configurable: true,
        get: () => {
          if (!(loaded.has(name) && loaded.get(name) < batch)) throw refused("PropertyNotLoaded");
          return read();
        },
      });
    }
    return object;
  };
  // A value that is readable once a sync has run since it was asked for.
  const afterSync = (object, name, read) => {
    const asked = batch;
    Object.defineProperty(object, name, {
      configurable: true,
      get: () => {
        if (!(asked < batch)) throw refused("ValueNotLoaded");
        return read();
      },
    });
    return object;
  };
  const missing = (code) =>
    queue(() => {
      throw refused(code);
    });
  const at = (address) => {
    const m = /^([A-E])([1-5])(?::([A-E])([1-5]))?$/.exec(address.replace(/\$/g, ""));
    if (!m) return null;
    const [c1, r1] = [m[1].charCodeAt(0) - 65, Number(m[2]) - 1];
    const [c2, r2] = m[3] ? [m[3].charCodeAt(0) - 65, Number(m[4]) - 1] : [c1, r1];
    return { c1, r1, c2, r2 };
  };
  // A source as the fake reads it: a range of Sheet1, or a name of either scope however it is prefixed.
  const resolve = (text) => {
    const bare = text.replace(/^=/, "").replace(/^(\[0\]!|Sheet1!)/, "");
    const name = state.names.find((item) => item.name === bare);
    const box = at(name ? name.formula.replace(/^=Sheet1!/, "") : bare);
    if (!box) return [];
    const values = [];
    for (let r = box.r1; r <= box.r2; r++) for (let c = box.c1; c <= box.c2; c++) values.push(state.cells[r][c]);
    return values.map(String);
  };
  const result = (compute) => afterSync({}, "value", compute);
  // A property that is read as `loadable` or `afterSync` made it, and written by a queued change.
  const writable = (object, name, write) => {
    const read = Object.getOwnPropertyDescriptor(object, name).get;
    Object.defineProperty(object, name, { get: read, set: (to) => queue(() => write(to)) });
    return object;
  };
  const rangeObject = (address) => {
    const box = at(address);
    const read = () => state.cells.slice(box.r1, box.r2 + 1).map((row) => row.slice(box.c1, box.c2 + 1));
    return writable(loadable({ address }, { values: read }), "values", (rows) => {
      host.lastChangeSync = host.syncs;
      host.changes.push(`cell ${address}=${rows[0][0]}`);
      while (state.cells[box.r1].length <= box.c1) state.cells[box.r1].push("");
      state.cells[box.r1][box.c1] = rows[0][0];
    });
  };
  const nameItem = (wanted, own, orNull) => {
    const find = () => state.names.find((item) => item.name === wanted && item.scope === own);
    if (!orNull && !find()) missing("ItemNotFound");
    const object = loadable({}, { formula: () => find().formula });
    afterSync(object, "isNullObject", () => !find());
    writable(object, "formula", (text) => {
      host.changes.push(`name ${own} ${wanted}=${text}`);
      find().formula = text;
    });
    object.delete = () =>
      queue(() => {
        host.changes.push(`delete ${own} ${wanted}`);
        state.names.splice(state.names.indexOf(find()), 1);
      });
    object.getRange = () => rangeObject(find() ? find().formula.replace(/^=Sheet1!/, "") : "?");
    return object;
  };
  const namesOf = (own) => {
    // An item's property is readable only when a load of the collection named it ("items/formula").
    const asked = new Set();
    const view = (item) => {
      const one = {};
      for (const key of Object.keys(item)) {
        Object.defineProperty(one, key, {
          get: () => {
            if (!asked.has(key)) throw refused("PropertyNotLoaded");
            return item[key];
          },
        });
      }
      return one;
    };
    const items = () => state.names.filter((item) => item.scope === own).map(view);
    const object = loadable({}, { items });
    const load = object.load;
    object.load = (names) => {
      for (const name of String(names).split(",")) if (name.includes("/")) asked.add(name.split("/")[1]);
      load(names);
    };
    return Object.assign(object, {
      getItem: (wanted) => nameItem(wanted, own, false),
      getItemOrNullObject: (wanted) => nameItem(wanted, own, true),
    });
  };
  const setSource = (dimension, call) => (to) => {
    host.calls.push(`${call} ${typeof to === "string" ? to : "range " + to.address}`);
    // A string this host does not take is refused when the call is sent, at the next sync, as Office.js does.
    if (typeof to === "string" && !strings) return missing("InvalidArgument");
    return queue(() => {
      host.changes.push(`${call} ${typeof to === "string" ? to : to.address}`);
      state.sources[dimension] = typeof to === "string" ? to : "Sheet1!" + to.address;
    });
  };
  const seriesObject = () =>
    Object.assign(loadable({}, { name: () => state.seriesName }), {
      getDimensionValues: (dimension) => result(() => resolve(state.sources[dimension])),
      getDimensionDataSourceString: (dimension) => result(() => state.sources[dimension]),
      getDimensionDataSourceType: () => result(() => "LocalRange"),
      setValues: setSource("Values", "setValues"),
      setXAxisValues: setSource("Categories", "setXAxisValues"),
    });
  const chartObject = {
    series: {
      getCount: () => result(() => series),
      getItemAt: (index) => {
        if (!(index < series)) missing("InvalidArgument");
        return seriesObject();
      },
    },
  };
  const sheetObject = (orNull) => {
    const object = {
      names: namesOf("Worksheet"),
      getRange: rangeObject,
      charts: {
        getCount: () => result(() => charts),
        getItemAt: (index) => {
          if (!(index < charts)) missing("InvalidArgument");
          return chartObject;
        },
      },
    };
    return orNull ? afterSync(object, "isNullObject", () => !sheet) : object;
  };
  const workbook = loadable({ names: namesOf("Workbook") }, { name: () => file });
  workbook.worksheets = {
    getItem: (name) => {
      if (name !== "Sheet1" || !sheet) missing("ItemNotFound");
      return sheetObject(false);
    },
    getItemOrNullObject: (name) => (name === "Sheet1" ? sheetObject(true) : afterSync({}, "isNullObject", () => true)),
  };
  const context = {
    workbook,
    sync: async () => {
      host.syncs += 1;
      const work = queued;
      queued = [];
      if (host.syncs === fail) throw refused("GeneralException");
      try {
        for (const one of work) one();
      } finally {
        batch += 1;
      }
    },
  };
  host.sandbox = {
    Excel: {
      run: async (work) => {
        await gate;
        await work(context);
        if (runRejects) throw refused("GeneralException");
      },
    },
    Office: {
      context: {
        platform: "OfficeOnline",
        diagnostics: { version: "fake" },
        requirements: { isSetSupported: (_api, set) => strings || set !== "1.21" },
      },
    },
    console: { log: (line) => host.lines.push(line) },
  };
  return host;
}

async function run(id, file, options) {
  const host = makeHost(file, options);
  vm.createContext(host.sandbox);
  vm.runInContext(source, host.sandbox);
  await vm.runInContext(`run(${JSON.stringify(id)})`, host.sandbox);
  assert.strictEqual(host.lines.length, 1, `${id} on ${file}: one line`);
  host.line = JSON.parse(host.lines[0]);
  assert.strictEqual(host.line.probe, "chart-series-name");
  assert.strictEqual(host.line.case, id);
  return host;
}

const RESET = "setValues range C2:C5";
const SET_CALLS = [
  RESET,
  "setValues range $B$2:$B$5",
  RESET,
  "setValues Sales",
  RESET,
  "setValues =[0]!Sales",
  RESET,
  "setValues =Sheet1!Sales",
  RESET,
  "setValues =Sales",
  "setXAxisValues range C2:C5",
  "setXAxisValues =Months",
];
const NOT_FRESH = "not a fresh series-control.xlsx: nothing changed";

async function main() {
  for (const file of Object.keys(FILES)) {
    const scope = FILES[file][0];
    // READ: everything is read, and the marker is the one change, made by the run's last sync.
    const read = await run("READ", file);
    assert.deepStrictEqual(read.line.cells, CELLS);
    assert.strictEqual(read.line.file, file);
    assert.deepStrictEqual(read.line.workbook, { sheet: true, charts: 1, series: 1 });
    assert.deepStrictEqual(read.line.env.sets, { 1.7: true, 1.12: true, 1.15: true, 1.21: true });
    const listed = [...read.line.names.workbook, ...read.line.names.sheet];
    assert.deepStrictEqual(listed.map((item) => [item.name, item.scope]), Object.keys(RANGES).map((n) => [n, scope]));
    assert.strictEqual(read.line.series.name, "stale");
    assert.deepStrictEqual(read.line.series["values:Values"], ["10", "20", "15", "25"]);
    assert.deepStrictEqual(read.line.series["values:Categories"], ["a", "b", "c", "d"]);
    assert.strictEqual(read.line.series["source:Values"], read.state.sources.Values);
    assert.deepStrictEqual(read.changes, ["cell E1=read"]);
    assert.strictEqual(read.lastChangeSync, read.syncs, "the marker is written by the run's last sync");
    assert.strictEqual(read.line.marker, "written");

    // CHANGE: the cell, then the name's formula, then the name, each in the name's own scope.
    const change = await run("CHANGE", file);
    assert.deepStrictEqual(change.changes, [
      "cell B2=11",
      `name ${scope} Sales==Sheet1!$C$2:$C$5`,
      `delete ${scope} Sales`,
    ]);
    assert.deepStrictEqual(change.line.cell.series["values:Values"], ["11", "20", "15", "25"]);
    const redefined = [...change.line.redefine.names.workbook, ...change.line.redefine.names.sheet];
    assert.strictEqual(redefined.find((item) => item.name === "Sales").formula, "=Sheet1!$C$2:$C$5");
    const left = [...change.line.delete.names.workbook, ...change.line.delete.names.sheet];
    assert.deepStrictEqual(left.map((item) => item.name), ["Label", "Months"]);
    for (const key of ["cell", "redefine", "delete"]) assert.strictEqual(change.line[key].change, "done");

    // With no name `Sales`: said twice, and only the cell is changed.
    const none = await run("CHANGE", file, { sales: false });
    assert.deepStrictEqual(none.changes, ["cell B2=11"]);
    assert.strictEqual(none.line.redefine.change, "no name Sales");
    assert.strictEqual(none.line.delete.change, "no name Sales");

    // A workbook without the probe's chart: one line that says what was found, nothing more read or changed.
    const others = [
      [{ sheet: false }, { sheet: false }],
      [{ charts: 0 }, { sheet: true, charts: 0 }],
      [{ charts: 2 }, { sheet: true, charts: 2 }],
      [{ series: 0 }, { sheet: true, charts: 1, series: 0 }],
      [{ series: 2 }, { sheet: true, charts: 1, series: 2 }],
    ];
    for (const [other, what] of others) {
      for (const id of ["READ", "CHANGE", "SET"]) {
        const elsewhere = await run(id, file, other);
        assert.deepStrictEqual(elsewhere.line.workbook, what);
        assert.match(elsewhere.line.error, /^no Sheet1 with one chart of one series/);
        assert.deepStrictEqual([elsewhere.changes, elsewhere.calls], [[], []]);
        assert.deepStrictEqual(Object.keys(elsewhere.line), ["probe", "case", "workbook", "error"]);
      }
    }

    // SET runs where the workbook's `Sales` is B2:B5 and B2 is 10, and nowhere else: not where the names are
    // the sheet's, not where `Sales` is gone, not after CHANGE's cell.
    const set = await run("SET", file);
    assert.strictEqual(set.line.error, scope === "Workbook" ? undefined : NOT_FRESH);
    if (scope !== "Workbook") assert.deepStrictEqual([set.changes, set.calls], [[], []]);
    const gone = await run("SET", file, { sales: false });
    assert.deepStrictEqual([gone.line.error, gone.changes, gone.calls], [NOT_FRESH, [], []]);
    const touched = await run("SET", file, { b2: 11 });
    assert.deepStrictEqual([touched.line.error, touched.changes, touched.calls], [NOT_FRESH, [], []]);

    // Any one sync may fail, and Excel.run may reject: still one line, and no change the full run does not make.
    for (const id of ["READ", "CHANGE", "SET"]) {
      const full = await run(id, file);
      for (let fail = 1; fail <= full.syncs; fail++) {
        const partial = await run(id, file, { fail });
        for (const made of partial.changes) assert.ok(full.changes.includes(made), `${id} sync ${fail}: ${made}`);
      }
      const rejected = await run(id, file, { runRejects: true });
      assert.strictEqual(rejected.line.failed.error, "GeneralException");
      assert.deepStrictEqual(rejected.changes, full.changes);
    }
  }

  // SET, on the control: the series is set to C2:C5 before each attempt, and the attempts come in order.
  const set = await run("SET", "series-control.xlsx");
  assert.deepStrictEqual(set.calls, SET_CALLS);
  for (const key of ["range of Sales", "values Sales", "values =[0]!Sales", "values =Sheet1!Sales", "values =Sales"]) {
    assert.deepStrictEqual(set.line[`reset before ${key}`].series["values:Values"], ["1", "2", "3", "4"], key);
    assert.deepStrictEqual(set.line[key].series["values:Values"], ["10", "20", "15", "25"], key);
  }
  assert.strictEqual(set.line["values =Sales"].series["source:Values"], "=Sales");
  assert.deepStrictEqual(set.line["reset before categories =Months"].series["values:Categories"], ["1", "2", "3", "4"]);
  assert.deepStrictEqual(set.line["categories =Months"].series["values:Categories"], ["a", "b", "c", "d"]);
  // Where strings are refused, the ranges still go through, each refusal is logged in its own step, and the
  // series still reads C2:C5 after it.
  const old = await run("SET", "series-control.xlsx", { strings: false });
  assert.deepStrictEqual(old.calls, SET_CALLS);
  assert.deepStrictEqual(old.changes.filter((made) => !made.endsWith("C2:C5")), ["setValues $B$2:$B$5"]);
  assert.strictEqual(old.line["range of Sales"].change, "done");
  for (const key of ["values Sales", "values =[0]!Sales", "values =Sheet1!Sales", "values =Sales"]) {
    assert.strictEqual(old.line[key].change.error, "InvalidArgument");
    assert.deepStrictEqual(old.line[key].series["values:Values"], ["1", "2", "3", "4"]);
  }
  assert.strictEqual(old.line["categories =Months"].change.error, "InvalidArgument");
  assert.strictEqual(old.line.env.sets["1.21"], false);

  // An unknown case: one line, and Excel is not asked.
  const unknown = await run("OTHER", "series-control.xlsx");
  assert.deepStrictEqual([unknown.line.error, unknown.syncs], ["unknown case", 0]);

  // In Script Lab: one button a case. A click logs its case's line and shows it under the buttons, and turns
  // off its own button and the ones before it: READ and CHANGE run in turn in one workbook, once each.
  const open = () => {
    const page = fakePage();
    const host = makeHost("series-control.xlsx");
    host.sandbox.document = page.document;
    vm.createContext(host.sandbox);
    vm.runInContext(source, host.sandbox);
    return { page, logged: () => host.lines };
  };
  const ids = ["READ", "CHANGE", "SET"];
  await checkButtons(open, ids, (line) => line.case, (i) => ids.map((_, j) => j <= i));
  // While a case runs every button is off, so a click on CHANGE cannot start it in the middle of READ.
  let release;
  const held = makeHost("series-control.xlsx", { gate: new Promise((resolve) => (release = resolve)) });
  const page = fakePage();
  held.sandbox.document = page.document;
  vm.createContext(held.sandbox);
  vm.runInContext(source, held.sandbox);
  const off = () => page.buttons.map((button) => button.disabled);
  const reading = page.buttons[0].onclick();
  assert.deepStrictEqual(off(), [true, true, true], "every button is off while READ runs");
  release();
  await reading;
  assert.deepStrictEqual(off(), [true, false, false], "READ stays off, and CHANGE and SET come back");
  await page.buttons[1].onclick();
  assert.deepStrictEqual(off(), [true, true, false], "after CHANGE, SET is the one left");
  assert.deepStrictEqual(held.lines.map((line) => JSON.parse(line).case), ["READ", "CHANGE"]);

  // README.md names what the script and the generator hold.
  for (const id of ["READ", "CHANGE", "SET"]) assert.ok(readme.includes("`" + id + "`"), `README names ${id}`);
  for (const spelling of ["Sales", "=[0]!Sales", "=Sheet1!Sales", "=Sales", "=Months"]) {
    assert.ok(readme.includes("`" + spelling + "`"), `README names ${spelling}`);
  }
  for (const file of Object.keys(FILES)) assert.ok(readme.includes("`" + file + "`"), `README names ${file}`);
  console.log(`ok: 3 cases on ${Object.keys(FILES).length} workbooks; every sync failed in turn; a button a case`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
