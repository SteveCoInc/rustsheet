// Checks cases.js without Excel: node check.js
//
// It runs every case against a small fake of Office.js that keeps a grid, the option and the calculation mode.
// It says nothing about Excel's values: those are what the request asks for. Checked here:
// - the script holds exactly the cases README.md names, and its table is the 99 rows README.md lists, row for
//   row;
// - generate.py writes the four files the README names, each with the sha256 the README gives;
// - every case logs exactly one line with no error, from a workbook with the option off and from one with it
//   on, and from a host in Manual mode, and leaves the option and the mode as it found them (SAVE alone leaves
//   the option on, when it ran to its end);
// - a workbook that holds a value is refused with nothing changed: no cell, no setting and no call, whatever
//   the mode. The same for a case id the script does not know and for a first sync that fails (review of
//   CR-321852193, round 1, B2). READ alone runs in a workbook with values, and changes nothing but a
//   calculation;
// - every table read holds the cells its column letter names, by their contents, in every snapshot: E reads A,
//   B, E and F, which are not next to each other (B1 of the same review);
// - a cell the API gives as -0 leaves other evidence than one it gives as 0 (B4), and every read carries the
//   kind of each value, which tells an error from a text that spells it;
// - IT closes its loop with iteration on at three iterations, with no early stop, and with the option on, and
//   it alone touches the iteration settings, which are put back as the option and the mode are; ST changes its
//   style's format after its cells have the style; W pastes formats after reading the cells once;
// - every array the script writes has the shape of its range, which Office.js requires;
// - the table cases write what README.md says: A writes the table with the option off, E with it on, each row's
//   format on its constant and its formula, General on the readers;
// - X turns the option on in Manual mode, F in Automatic mode;
// - the option and the mode are put back when a sync fails, whichever one, whether the failed sync had applied
//   its writes or not. One combination cannot be undone by a script and is left out: the last sync, which is
//   the one that puts them back, failing with its writes dropped.
const assert = require("assert");
const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const vm = require("vm");
const { fakePage, checkButtons } = require("../script_lab_page");

const src = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");

const columnNumber = (letters) => [...letters].reduce((n, ch) => n * 26 + ch.charCodeAt(0) - 64, 0);
const columnLetters = (n) =>
  (n > 26 ? columnLetters(Math.floor((n - 1) / 26)) : "") + String.fromCharCode(65 + ((n - 1) % 26));
// "A2:D96", "G1" -> { row, col, rows, cols }; a whole column ("A:A") -> null.
function box(address) {
  const m = /^([A-Z]+)(\d+)(?::([A-Z]+)(\d+))?$/.exec(address);
  if (!m) return null;
  const [row, col] = [Number(m[2]), columnNumber(m[1])];
  const [row2, col2] = m[3] ? [Number(m[4]), columnNumber(m[3])] : [row, col];
  return { row, col, rows: row2 - row + 1, cols: col2 - col + 1 };
}

// `used`: the workbook holds a value before the script runs. `fail`: the sync with that number (from 1) rejects
// once, after applying the queued settings (`applied`) or dropping them.
const ITERATION_OFF = { enabled: false, maxIteration: 100, maxChange: 0.001 };
// `mode`: the calculation mode the host is in. `iteration`: its iterative-calculation settings. `negativeZero`:
// a cell whose value the API gives as -0.
function makeHost(options = {}) {
  const { pad = false, mode = "Automatic", iteration = ITERATION_OFF } = options;
  const { used = false, negativeZero, fail } = options;
  const state = { pad, mode, iteration: { ...iteration } };
  const host = { syncs: 0, state, timeline: [], cells: new Map() };
  let queued = [];
  const sync = async () => {
    host.syncs += 1;
    const failing = fail && host.syncs === fail.sync;
    if (!failing || fail.applied) for (const apply of queued) apply();
    queued = [];
    if (failing) throw Object.assign(new Error("injected sync failure"), { code: "Injected" });
  };
  const cell = (key) => host.cells.get(key) || {};
  function range(address) {
    const b = box(address);
    const grid = (pick) =>
      Array.from({ length: b.rows }, (_, i) =>
        Array.from({ length: b.cols }, (_unused, j) => {
          const key = `${columnLetters(b.col + j)}${b.row + i}`;
          return pick(cell(key), key);
        }),
      );
    const write = (kind, m) => {
      assert.ok(b, `${address}: ${kind} written to a range with no size`);
      const row = (r) => Array.isArray(r) && r.length === b.cols;
      const shape = Array.isArray(m) && m.length === b.rows && m.every(row);
      assert.ok(shape, `${address}: the ${kind} array is not ${b.rows} x ${b.cols}`);
      m.forEach((r, i) =>
        r.forEach((v, j) => {
          const key = `${columnLetters(b.col + j)}${b.row + i}`;
          const old = cell(key);
          // A format is kept beside what the cell holds; a value or a formula replaces what it held.
          const held = kind === "numberFormat" ? old : { numberFormat: old.numberFormat };
          host.cells.set(key, { ...held, [kind]: v });
        }),
      );
      host.timeline.push({ write: kind, address, ...state });
    };
    const held = (c) => (c.formulas !== undefined ? c.formulas : c.values !== undefined ? c.values : "");
    return {
      load() {
        host.timeline.push({ read: address, ...state });
      },
      get values() {
        const shown = (c) => (c.formulas !== undefined ? `<${c.formulas}>` : held(c));
        return grid((c, key) => (key === negativeZero ? -0 : shown(c)));
      },
      set values(m) {
        write("values", m);
      },
      // The kind of what the fake gives as a value: it does not calculate, so a formula cell is a String.
      get valueTypes() {
        const kind = { number: "Double", boolean: "Boolean", string: "String" };
        return this.values.map((row) => row.map((v) => (v === "" ? "Empty" : kind[typeof v])));
      },
      get formulas() {
        return grid(held);
      },
      set formulas(m) {
        write("formulas", m);
      },
      get text() {
        return grid((c) => String(held(c)));
      },
      get numberFormat() {
        return grid((c) => c.numberFormat || "General");
      },
      set numberFormat(m) {
        write("numberFormat", m);
      },
      set style(name) {
        host.timeline.push({ write: "style", address, name, ...state });
      },
      copyFrom(source, type) {
        host.timeline.push({ write: "copyFrom", address, source, type, ...state });
      },
      format: {
        set columnWidth(v) {
          host.timeline.push({ write: "columnWidth", address, value: v, ...state });
        },
        fill: {
          set color(v) {
            host.timeline.push({ write: "fill", address, value: v, ...state });
          },
        },
      },
      conditionalFormats: {
        add(type) {
          host.timeline.push({ write: "conditionalFormat", address, type, ...state });
          return { custom: { rule: {}, format: {} } };
        },
      },
    };
  }
  const sheet = {
    getRange: range,
    getUsedRangeOrNullObject: () => ({ load() {}, isNullObject: !used, address: "Sheet1!A1:B2" }),
  };
  const workbook = {
    load() {},
    get usePrecisionAsDisplayed() {
      return state.pad;
    },
    set usePrecisionAsDisplayed(v) {
      host.timeline.push({ set: "pad", value: v, ...state });
      queued.push(() => {
        state.pad = v;
      });
    },
    worksheets: { load() {}, items: [sheet], getActiveWorksheet: () => sheet },
    styles: {
      add(name) {
        host.timeline.push({ write: "addStyle", name, ...state });
      },
      getItem: (name) => ({
        set numberFormat(v) {
          host.timeline.push({ write: "styleFormat", name, value: v, ...state });
        },
      }),
    },
  };
  // The three iteration settings, each written at the next sync, as the option and the mode are.
  const iterativeCalculation = { load() {} };
  for (const key of ["enabled", "maxIteration", "maxChange"]) {
    Object.defineProperty(iterativeCalculation, key, {
      get: () => state.iteration[key],
      set(v) {
        host.timeline.push({ set: `iteration.${key}`, value: v, ...state });
        queued.push(() => {
          state.iteration = { ...state.iteration, [key]: v };
        });
      },
    });
  }
  const application = {
    load() {},
    iterativeCalculation,
    calculationState: "Done",
    get calculationMode() {
      return state.mode;
    },
    set calculationMode(v) {
      host.timeline.push({ set: "mode", value: v, ...state });
      queued.push(() => {
        state.mode = v;
      });
    },
    calculate(type) {
      host.timeline.push({ call: "calculate", type, ...state });
    },
  };
  host.context = { workbook, application, sync };
  return host;
}

// The script loaded for `host`. With a `page` it finds Script Lab's page and puts its buttons on it.
function load(host, lines, page) {
  const sandbox = {
    document: page ? page.document : undefined,
    console: { log: (line) => lines.push(JSON.parse(line)) },
    Office: { context: { diagnostics: { version: "16.0.FAKE", platform: "OfficeOnline" } } },
    Excel: {
      run: async (f) => f(host.context),
      CalculationMode: { manual: "Manual", automatic: "Automatic" },
      CalculationType: { recalculate: "Recalculate", full: "Full" },
      RangeCopyType: { values: "Values", all: "All", formats: "Formats" },
      ConditionalFormatType: { custom: "Custom" },
    },
  };
  vm.createContext(sandbox);
  const exports = "globalThis.__run = run; globalThis.__ids = Object.keys(CASES); globalThis.__T = T;";
  vm.runInContext(`${src}\n;${exports}`, sandbox);
  return sandbox;
}

async function runCase(id, options) {
  const host = makeHost(options);
  const lines = [];
  const sandbox = load(host, lines);
  let rejected = null;
  await sandbox.__run(id).catch((e) => {
    rejected = e; // only the sync of run()'s own cleanup can make run() reject
  });
  return { host, lines, ids: sandbox.__ids, table: sandbox.__T, rejected };
}

const writes = (host) => host.timeline.filter((e) => e.write);
const padSets = (host) => host.timeline.filter((e) => e.set === "pad");
const modeSets = (host) => host.timeline.filter((e) => e.set === "mode");
const iterationSets = (host) => host.timeline.filter((e) => e.set && e.set.startsWith("iteration."));

(async () => {
  const first = await runCase("K");
  const ids = Array.from(first.ids);
  const table = Array.from(first.table, (row) => Array.from(row)); // arrays of this realm, to compare with
  const last = table.length + 1;
  // Written out, not taken from the script: a case or a row dropped from it must fail here, not shorten the
  // run.
  const cases = "A E F X FMT O S K W G CF I ST IT SAVE READ".split(" ");
  assert.deepStrictEqual(ids, cases, "the script holds the README's cases");
  assert.strictEqual(table.length, 99, "the table has the 99 rows the README says");
  // The README lists every row ("The rows of the table"). The script's table must be that list, row for row.
  const readme = fs.readFileSync(path.join(__dirname, "README.md"), "utf8");
  const listed = [...readme.matchAll(/^\| (\d+) \| `(.*)` \| (\S+) \| `(.*)` \|$/gm)];
  assert.strictEqual(listed.length, table.length, "the README lists as many rows as the table has");
  listed.forEach(([, sheetRow, format, number, formula], i) => {
    const row = table[i];
    const where = `row ${i + 2} of the table`;
    assert.strictEqual(Number(sheetRow), i + 2, `${where}: its place in the README's list`);
    assert.strictEqual(row[0], format, `${where}: the format the README lists`);
    assert.strictEqual(JSON.stringify(row[1]), number, `${where}: the number the README lists`);
    assert.strictEqual(row[2] || `=${row[1]}`, formula, `${where}: the formula the README lists`);
  });

  // The hosts a case is run from: the option off or on, in Automatic or Manual mode, and one with iterative
  // calculation on under other limits than the script's.
  const ITERATION_ON = { enabled: true, maxIteration: 50, maxChange: 0.01 };
  const hosts = [
    { pad: false, mode: "Automatic", iteration: ITERATION_OFF },
    { pad: true, mode: "Automatic", iteration: ITERATION_OFF },
    { pad: false, mode: "Manual", iteration: ITERATION_OFF },
    { pad: true, mode: "Manual", iteration: ITERATION_OFF },
    { pad: false, mode: "Automatic", iteration: ITERATION_ON },
  ];
  const onOff = (flag) => (flag ? "on" : "off");
  const named = ({ pad, mode, iteration }) =>
    `option ${onOff(pad)}, ${mode} mode, iteration ${onOff(iteration.enabled)}`;
  // A run that must change nothing: no cell, no setting, no call, and the host as it was.
  const untouched = (host, start, where) => {
    assert.deepStrictEqual(host.timeline, [], `${where}: nothing is written, set or called`);
    assert.deepStrictEqual(host.state, start, `${where}: the host is as it was`);
  };

  // The files generate.py writes are the ones the README names: every sha256 it prints is in the README beside
  // its file's name, and the README names no other file.
  {
    const out = fs.mkdtempSync(path.join(os.tmpdir(), "pad-probe-"));
    const run = spawnSync("python3", ["-I", path.join(__dirname, "generate.py"), out], { encoding: "utf8" });
    fs.rmSync(out, { recursive: true });
    assert.strictEqual(run.status, 0, `generate.py: ${run.stderr}`);
    const written = run.stdout.trim().split("\n").map((line) => line.split(" "));
    assert.strictEqual(written.length, 4, "generate.py writes four files");
    for (const [sha, name] of written) {
      const named = new RegExp("`" + name.replace(".", "\\.") + "`\\s+\\(sha256\\s+`" + sha + "`\\)");
      assert.ok(named.test(readme), `the README names ${name} with the sha256 generate.py prints`);
    }
    const inReadme = new Set(readme.match(/pad-[a-z-]+\.xlsx/g));
    assert.deepStrictEqual([...inReadme].sort(), written.map(([, name]) => name).sort(), "the README's files");
  }

  for (const id of ids) {
    const opened = id === "READ";
    for (const start of hosts) {
      const { pad, mode, iteration } = start;
      const { host, lines, rejected } = await runCase(id, { ...start, used: opened });
      const where = `${id}, ${named(start)} at start`;
      assert.ok(!rejected, `${where}: run() does not reject`);
      assert.strictEqual(lines.length, 1, `${where}: one line`);
      assert.strictEqual(lines[0].id, id);
      assert.ok(!("error" in lines[0]), `${where}: ${lines[0].error} ${lines[0].message}`);
      assert.strictEqual(lines[0].padAtStart, pad, `${where}: logs the option it found`);
      assert.strictEqual(lines[0].modeAtStart, mode, `${where}: logs the mode it found`);
      assert.strictEqual(host.state.mode, mode, `${where}: the mode afterwards`);
      assert.strictEqual(host.state.pad, id === "SAVE" ? true : pad, `${where}: the option afterwards`);
      assert.deepStrictEqual(lines[0].iterationAtStart, iteration, `${where}: logs the iteration settings`);
      assert.deepStrictEqual(host.state.iteration, iteration, `${where}: the iteration settings afterwards`);
      if (id !== "IT") assert.deepStrictEqual(iterationSets(host), [], `${where}: iteration is not touched`);
      if (opened) {
        assert.deepStrictEqual(writes(host), [], `${where}: READ writes nothing`);
        assert.deepStrictEqual(padSets(host), [], `${where}: READ never sets the option`);
        assert.deepStrictEqual(modeSets(host), [], `${where}: READ never sets the mode`);
        continue;
      }
      // Every case starts in Automatic mode: nothing is written to the grid in another.
      for (const e of writes(host)) {
        if (id === "X") continue;
        assert.strictEqual(e.mode, "Automatic", `${where}: ${e.write} of ${e.address} in Automatic`);
      }
      // Every case starts from full precision: nothing is written to the grid before the option is off.
      const first = writes(host)[0];
      const firstSet = padSets(host)[0];
      if (pad) assert.strictEqual(firstSet.value, false, `${where}: the option is turned off first`);
      const before = host.timeline.indexOf(firstSet) < host.timeline.indexOf(first);
      if (pad) assert.ok(before, `${where}: before any write`);
    }
    // A workbook that holds a value is refused, and a first sync that fails ends the run: nothing changes.
    for (const start of hosts) {
      if (!opened) {
        const { host, lines } = await runCase(id, { ...start, used: true });
        assert.strictEqual(lines.length, 1, `${id}, used workbook: one line`);
        assert.strictEqual(lines[0].error, "WorkbookNotEmpty", `${id}: a workbook with values is refused`);
        untouched(host, start, `${id}, used workbook, ${named(start)}`);
      }
      for (const applied of [false, true]) {
        const { host, lines } = await runCase(id, { ...start, used: opened, fail: { sync: 1, applied } });
        assert.strictEqual(lines[0].error, "Injected", `${id}: a failed first sync is logged`);
        untouched(host, start, `${id}, first sync fails, ${named(start)}`);
      }
    }
  }
  // A case id the script does not know.
  for (const start of hosts) {
    const { host, lines } = await runCase("NOPE", start);
    assert.strictEqual(lines.length, 1, "an unknown case: one line");
    assert.strictEqual(lines[0].error, "UnknownCase", "an unknown case is refused");
    untouched(host, start, `an unknown case, ${named(start)}`);
  }

  // The table cases write what the README says.
  for (const [id, optionAtWrite, columns] of [
    ["A", false, { constants: "A", formulas: "B", readers: ["C", "D"] }],
    ["E", true, { constants: "A", formulas: "B", readers: [] }],
    ["E", true, { constants: "E", formulas: "F", readers: [] }],
  ]) {
    const { host, lines } = await runCase(id);
    const tableWrites = writes(host).filter((e) => ["values", "formulas", "numberFormat"].includes(e.write));
    assert.ok(tableWrites.length > 0, `${id}: writes the table`);
    for (const e of tableWrites) {
      assert.strictEqual(e.pad, optionAtWrite, `${id}: ${e.write} of ${e.address} with the option`);
    }
    table.forEach(([format, number, formula], i) => {
      const row = i + 2;
      const constant = host.cells.get(`${columns.constants}${row}`);
      const computed = host.cells.get(`${columns.formulas}${row}`);
      const [a, b] = [`${columns.constants}${row}`, `${columns.formulas}${row}`];
      assert.strictEqual(constant.values, number, `${id}: ${a} holds the row's number`);
      assert.strictEqual(constant.numberFormat, format, `${id}: ${a} has the row's format`);
      assert.strictEqual(computed.formulas, formula || `=${number}`, `${id}: ${b} holds the formula`);
      assert.strictEqual(computed.numberFormat, format, `${id}: ${b} has the row's format`);
      columns.readers.forEach((c, j) => {
        const reader = host.cells.get(`${c}${row}`);
        assert.strictEqual(reader.formulas, `=${[a, b][j]}`, `${id}: ${c}${row}`);
        assert.strictEqual(reader.numberFormat, "General", `${id}: ${c}${row} is General`);
      });
    });
    assert.ok(!host.cells.has(`A${last + 1}`), `${id}: nothing below the table`);
    if (id === "A") assert.strictEqual(lines[0].data.rows.length, table.length, "A logs its rows");
  }
  // Every table read holds, under each column letter, that column's own cells: what the fake gives for a
  // constant is its number, and for a formula its text between angle brackets.
  for (const [id, snapshots, columns] of [
    ["A", ["off", "on", "onAfterFull", "onAfterSecondFull"], { A: "number", B: "formula", C: "=A", D: "=B" }],
    [
      "E",
      ["entered", "formatted", "afterFull", "afterSecondFull"],
      { A: "number", B: "formula", E: "number", F: "formula" },
    ],
  ]) {
    const { host, lines } = await runCase(id);
    const expected = (kind, [, number, formula], row) =>
      kind === "number" ? number : `<${kind === "formula" ? formula || `=${number}` : `${kind}${row}`}>`;
    const reads = Object.keys(lines[0].data).filter((k) => k !== "rows");
    assert.deepStrictEqual(reads, snapshots, `${id}: its reads`);
    // Two reads, then a full calculation before each of the last two: a snapshot is not the one before it.
    const steps = host.timeline.filter((e) => e.read || e.call).map((e) => (e.read ? "read" : e.type));
    const order = ["read", "read", "Full", "read", "Full", "read"];
    assert.deepStrictEqual(steps, order, `${id}: reads and calculations`);
    for (const name of snapshots) {
      const read = lines[0].data[name];
      const letters = Object.keys(read).filter((k) => k.length === 1);
      assert.deepStrictEqual(letters, Object.keys(columns), `${id} ${name}: the columns read`);
      for (const [c, kind] of Object.entries(columns)) {
        const values = table.map((row, i) => expected(kind, row, i + 2));
        assert.deepStrictEqual(read[c].values, values, `${id} ${name}: column ${c} holds column ${c}'s cells`);
        const kinds = values.map((v) => (typeof v === "number" ? "Double" : "String"));
        assert.deepStrictEqual(read[c].valueTypes, kinds, `${id} ${name}: column ${c}'s value kinds`);
        const text = values.map((v) => String(v).replace(/^<|>$/g, ""));
        assert.deepStrictEqual(read[c].text, text, `${id} ${name}: column ${c}'s text`);
        assert.deepStrictEqual(read[c].negativeZeros, [], `${id} ${name}: column ${c} lists no -0`);
      }
    }
  }
  // The other cases: each snapshot is a read of the range the README names, and holds that range's cells.
  const snapshotsOf = {
    F: ["before", "on", "onAfterF9", "narrowed", "narrowedAfterF9", "narrowedAfterFull"]
      .concat(["widened", "widenedAfterF9", "widenedAfterFull"])
      .map((name) => [name, "A1:D1"]),
    O: ["on", "off", "offAfterF9", "offAfterFull"].map((name) => [name, "A1:D1"]),
    FMT: ["offStart", "offNoEdit", "offConstantFormat", "offFormulaFormat", "offEmptyFormat", "offFill"]
      .concat(["offValueEdit", "offAfterF9"])
      .map((name) => [name, "B1:F3"]),
    S: ["off", "on", "onAfterFull"].map((name) => [name, "A1:C4"]),
    K: ["off", "on"].map((name) => [name, "A1:C8"]),
    W: [["written", "A1:A5"], ["source", "G1"], ["beforeFormats", "A6:B6"], ["afterFormats", "A6:B6"]],
    G: ["off", "on", "onAfterFull"].map((name) => [name, "A1:C3"]),
    CF: ["off", "on", "onAfterF9", "onAfterFull"].map((name) => [name, "A1:C4"]),
    I: ["off", "on", "onAfterFull"].map((name) => [name, "A1:C2"]),
    ST: ["on", "styled", "styledAfterF9", "styledAfterFull"].map((name) => [name, "A1:D1"]),
    IT: ["entered", "afterF9", "afterSecondF9"].map((name) => [name, "A1:B1"]),
    SAVE: [["on", "A1:C1"]],
    READ: ["opened", "afterF9", "afterFull"].map((name) => [name, "A1:D1"]),
  };
  snapshotsOf.X = snapshotsOf.F;
  for (const id of ids.filter((id) => id !== "A" && id !== "E")) {
    const { host, lines } = await runCase(id, { used: id === "READ" });
    const expected = snapshotsOf[id];
    assert.ok(expected, `${id}: its reads are listed here`);
    // A snapshot is a key whose value holds `values`; IT also logs the limits it set.
    const taken = Object.keys(lines[0].data).filter((key) => "values" in lines[0].data[key]);
    assert.deepStrictEqual(taken, expected.map(([name]) => name), `${id}: its snapshots`);
    const reads = host.timeline.filter((e) => e.read).map((e) => e.read);
    assert.deepStrictEqual(reads, expected.map(([, address]) => address), `${id}: the ranges it reads`);
    for (const [name, address] of expected) {
      const range = host.context.workbook.worksheets.getActiveWorksheet().getRange(address);
      const held = JSON.parse(JSON.stringify(range.values)); // as the line carries them
      // FMT writes B1 a second time, after its format changes: the snapshots before that hold the first value.
      const names = expected.map(([snapshot]) => snapshot);
      if (id === "FMT" && names.indexOf(name) < names.indexOf("offValueEdit")) held[0][0] = 1.5;
      assert.deepStrictEqual(lines[0].data[name].values, held, `${id} ${name}: holds the cells of ${address}`);
      const kinds = lines[0].data[name].valueTypes;
      assert.deepStrictEqual(kinds, range.valueTypes, `${id} ${name}: with their kinds`);
      // A snapshot is taken in the state its name says: the option is off in "before" and in every "off…".
      const optionOn = id !== "READ" && !/^(before$|off)/.test(name);
      assert.strictEqual(lines[0].data[name].pad, optionOn, `${id} ${name}: read with the option as named`);
      const manual = id === "X" && name !== "before";
      assert.strictEqual(lines[0].data[name].mode, manual ? "Manual" : "Automatic", `${id} ${name}: its mode`);
    }
  }
  // Signed zero: a cell the API gives as -0 is listed, in a table read and in a plain read, and a 0 is not.
  {
    const table12 = (await runCase("A", { negativeZero: "B12" })).lines[0].data;
    for (const name of ["off", "on", "onAfterFull", "onAfterSecondFull"]) {
      assert.deepStrictEqual(table12[name].B.negativeZeros, [12], `A ${name}: B12 is listed as -0`);
      assert.strictEqual(table12[name].B.values[10], 0, `A ${name}: JSON alone shows 0 for B12`);
      assert.deepStrictEqual(table12[name].A.negativeZeros, [], `A ${name}: column A lists none`);
    }
    // K writes -0 to A8 itself, and the fake gives back what it holds: that control is listed, and A7 too
    // when the host gives -0 for it.
    const plain = (await runCase("K", { negativeZero: "A7" })).lines[0].data;
    assert.deepStrictEqual(plain.on.negativeZeros, [[6, 0], [7, 0]], "K: A7 and A8, by their place in A1:C8");
    assert.deepStrictEqual((await runCase("K")).lines[0].data.on.negativeZeros, [[7, 0]], "K: A8 alone");
  }
  // In E, columns E and F get their numbers before their formats, and A and B after.
  {
    const { host } = await runCase("E");
    const order = (kind, col) =>
      host.timeline.findIndex((e) => e.write === kind && e.address.startsWith(`${col}2:`));
    assert.ok(order("numberFormat", "A") < order("values", "A"), "E: A is formatted, then written");
    assert.ok(order("values", "E") < order("numberFormat", "E"), "E: E is written, then formatted");
    assert.ok(order("formulas", "F") < order("numberFormat", "F"), "E: F is written, then formatted");
  }
  // F turns the option on in Automatic mode and X in Manual mode; both change the formats with it on.
  for (const [id, mode] of [
    ["F", "Automatic"],
    ["X", "Manual"],
  ]) {
    const { host } = await runCase(id, { mode: "Manual" });
    assert.strictEqual(host.state.mode, "Manual", `${id}: a host in Manual mode is left in Manual mode`);
    const on = padSets(host).find((e) => e.value === true);
    assert.strictEqual(on.mode, mode, `${id}: the option is turned on in ${mode} mode`);
    const formats = writes(host).filter((e) => e.write === "numberFormat" && e.address === "A1:B1");
    assert.strictEqual(formats.length, 3, `${id}: three formats on A1:B1`);
    assert.deepStrictEqual(
      formats.map((e) => [e.pad, e.mode]),
      [[false, "Automatic"], [true, mode], [true, mode]],
      `${id}: the second and third format are set with the option on`,
    );
  }

  // IT closes its loop with iteration on at three iterations and no early stop, and with the option on.
  for (const start of hosts) {
    const { host, lines } = await runCase("IT", start);
    const loop = writes(host).find((e) => e.write === "formulas" && e.address === "A1:B1");
    assert.ok(loop, "IT writes its loop");
    const limits = { enabled: true, maxIteration: 3, maxChange: 1e-9 };
    assert.deepStrictEqual(loop.iteration, limits, `IT, ${named(start)}: the loop is closed under its limits`);
    assert.strictEqual(loop.pad, true, `IT, ${named(start)}: the loop is closed with the option on`);
    assert.deepStrictEqual(lines[0].data.iteration, limits, `IT, ${named(start)}: logs the limits it set`);
  }
  // ST changes the format through the style, with the option on, after the cells have the style.
  {
    const { host } = await runCase("ST");
    const order = ["addStyle", "styleFormat", "style", "styleFormat"];
    const steps = writes(host).filter((e) => order.includes(e.write));
    assert.deepStrictEqual(steps.map((e) => e.write), order, "ST: the style, its cells, then its new format");
    const option = steps.map((e) => e.pad);
    assert.deepStrictEqual(option, [false, false, false, true], "ST: the last with the option on");
    assert.strictEqual(steps[3].value, "0.00", "ST: the style's format becomes 0.00");
  }
  // FMT never turns the option on, changes four formats one at a time, each between two reads, and only then
  // a value.
  {
    const { host } = await runCase("FMT");
    assert.deepStrictEqual(padSets(host), [], "FMT: the option is not touched");
    const events = host.timeline.filter((e) => e.read || e.write);
    const steps = events.map((e) => (e.read ? "read" : `${e.write} ${e.address}`));
    const after = steps.slice(steps.indexOf("read"));
    assert.deepStrictEqual(
      after,
      ["read", "read", "numberFormat B1", "read", "numberFormat B2", "read", "numberFormat B3:D3", "read"]
        .concat(["fill B1", "read", "values B1", "read", "read"]),
      "FMT: one change between two reads, the value last",
    );
  }
  // W pastes formats onto A6 and B6 after it has read them once.
  {
    const { host } = await runCase("W");
    const pastes = host.timeline.filter((e) => e.write === "copyFrom" && e.type === "Formats");
    assert.deepStrictEqual(pastes.map((e) => e.address), ["A6", "B6"], "W: formats pasted onto A6 and B6");
    const firstRead = host.timeline.findIndex((e) => e.read === "A6:B6");
    assert.ok(firstRead < host.timeline.indexOf(pastes[0]), "W: A6:B6 is read before the paste of formats");
  }

  // Every case under one failed sync.
  let failures = 0;
  for (const id of ids) {
    const opened = id === "READ";
    for (const start of hosts) {
      const { pad, mode, iteration } = start;
      const total = (await runCase(id, { ...start, used: opened })).host.syncs;
      for (let n = 1; n <= total; n++) {
        for (const applied of [false, true]) {
          if (n === total && !applied) continue; // the writes that put the settings back are themselves dropped
          const fail = { sync: n, applied };
          const { host, lines, rejected } = await runCase(id, { ...start, used: opened, fail });
          const failure = `sync ${n} of ${total} fails, writes ${applied ? "applied" : "dropped"}`;
          const where = `${id}, ${named(start)}, ${failure}`;
          const kept = id === "SAVE" && n === total; // SAVE ran to its end: the option stays on, as it says
          assert.strictEqual(host.state.pad, kept ? true : pad, `${where}: the option is put back`);
          assert.strictEqual(host.state.mode, mode, `${where}: the mode is put back`);
          assert.deepStrictEqual(host.state.iteration, iteration, `${where}: iteration is put back`);
          if (n < total) {
            assert.ok(!rejected, `${where}: run() handles the failure`);
            assert.strictEqual(lines.length, 1, `${where}: one line`);
            assert.ok("error" in lines[0], `${where}: the line carries the error`);
          }
          failures += 1;
        }
      }
    }
  }
  // In Script Lab: one button a case. A click logs its case's line and shows it under the buttons, and turns
  // the buttons off.
  const open = () => {
    const page = fakePage();
    const shown = [];
    load(makeHost(), shown, page);
    return { page, logged: () => shown.map((line) => JSON.stringify(line)) };
  };
  await checkButtons(open, ids, (line) => line.id);
  const each = `${ids.length} cases, ${table.length} table rows`;
  console.log(`ok: ${each}, each from ${hosts.length} hosts; ${failures} failed syncs; a button a case`);
})().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
