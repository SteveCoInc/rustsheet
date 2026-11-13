// Checks cases.js without Excel: node check.js
//
// It runs the script against a small fake of Office.js that queues what the script asks and does it at the next
// sync, as Office.js does. The fake makes every chart it is asked for and reads back made-up values, so it says
// nothing about Excel: what Excel does is what the request asks. Checked here:
// - a run in a fresh workbook logs an ENV line, one line a case in README.md's order, and an END line;
// - each case's sheet is named by the case and holds README.md's range; each chart case's chart has README.md's
//   type, the case's name, the whole range as its source and a place beside the data; the R cases have no chart;
// - the data are what README.md says of them (the histogram values, the stock columns, the twelve selections);
// - a chart the host refuses is that case's answer, with the error's code, and the cases after it still run;
// - a host with no data table, and one with no bin options, give that answer in the case's line and no other;
// - a workbook that already has a sheet named as a case is refused: one line, nothing written;
// - any one sync may fail: the run still ends without an exception, no sheet is added twice and no chart is added
//   to a sheet twice; if the first sync fails nothing is written;
// - Scott's rule gives the bin widths, edges and counts README.md's table shows for its four readings;
// - in Script Lab the script puts one button on the page, which runs it, shows its lines and is then off.
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vm = require("vm");
const { fakePage, checkButtons } = require("../script_lab_page");

const source = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");
const readme = fs.readFileSync(path.join(__dirname, "README.md"), "utf8");

function refused(code) {
  return Object.assign(new Error(`the fake host: ${code}`), { code });
}

// ---- The fake host ---------------------------------------------------------------------------------------------
// `refuse`: chart types the host does not make. `lacks`: "dataTable" (the chart gives a null object) or
// "binOptions" (loading them fails). `fail`: the sync, counted from 1, that fails and loses its batch.
// `existing`: the workbook's sheets before the run.
function makeHost({ refuse = [], lacks = [], fail = 0, existing = ["Sheet1"] } = {}) {
  const host = { lines: [], sheets: new Map(), syncs: 0, added: [], charted: [] };
  for (const name of existing) host.sheets.set(name, { cells: new Map(), formats: new Map(), charts: [] });
  let queue = [];
  const later = (act) => queue.push(act);

  // An object whose loaded properties are made up, except the ones the script set.
  const loadable = (set, lacking) => {
    const object = {
      isNullObject: false,
      load: (names) =>
        later(() => {
          if (lacking) throw refused("ApiNotFound");
          for (const name of names.split(",")) object[name] = name in set ? set[name] : `fake ${name}`;
        }),
    };
    return object;
  };
  const setter = (object, set, name) =>
    Object.defineProperty(object, name, {
      set: (value) => later(() => (set[name] = value)),
      get: () => set[name],
      configurable: true,
    });

  const chartOf = (record) => {
    const made = () => {
      if (!record.made) throw refused("InvalidObjectPath");
    };
    const chart = loadable({ chartType: record.type });
    Object.defineProperty(chart, "name", { set: (value) => later(() => made() || (record.name = value)) });
    chart.setPosition = (from, to) => later(() => made() || (record.position = [from, to]));
    chart.series = {
      getItemAt: (index) => {
        const set = {};
        const series = loadable(set);
        for (const name of ["axisGroup", "filtered"]) setter(series, set, name);
        const load = series.load;
        series.load = (names) => {
          later(() => {
            made();
            if (index >= record.series) throw refused("InvalidArgument");
          });
          load(names);
        };
        later(() => made());
        series.binOptions = loadable({}, lacks.includes("binOptions"));
        record.touched.push([index, set]);
        return series;
      },
    };
    chart.getDataTableOrNullObject = () => {
      const set = {};
      const table = loadable(set);
      setter(table, set, "visible");
      if (lacks.includes("dataTable")) later(() => (table.isNullObject = true));
      record.table = set;
      return table;
    };
    return chart;
  };

  const sheetOf = (name) => ({
    getRange: (address) => {
      const range = {};
      for (const [property, store] of [["values", "cells"], ["numberFormat", "formats"]]) {
        Object.defineProperty(range, property, {
          set: (value) =>
            later(() => {
              const sheet = host.sheets.get(name);
              if (!sheet) throw refused("ItemNotFound");
              sheet[store].set(address, value);
            }),
        });
      }
      range.address = address;
      return range;
    },
    charts: {
      add: (type, range, seriesBy) => {
        const record = { type, range: range.address, seriesBy, made: false, touched: [], series: 0 };
        later(() => {
          const sheet = host.sheets.get(name);
          if (!sheet) throw refused("ItemNotFound");
          if (refuse.includes(type)) throw refused("InvalidArgument");
          const data = sheet.cells.get(range.address);
          record.series = Math.max(1, (data ? data[0].length : 2) - 1);
          record.made = true;
          sheet.charts.push(record);
          host.charted.push(name);
        });
        return chartOf(record);
      },
    },
  });

  const worksheets = {
    items: [],
    load: () => later(() => (worksheets.items = [...host.sheets.keys()].map((name) => ({ name })))),
    add: (name) => {
      later(() => {
        if (host.sheets.has(name)) throw refused("ItemAlreadyExists");
        host.sheets.set(name, { cells: new Map(), formats: new Map(), charts: [] });
        host.added.push(name);
      });
      return sheetOf(name);
    },
  };
  const context = {
    workbook: { worksheets },
    sync: async () => {
      host.syncs += 1;
      const batch = queue;
      queue = [];
      if (host.syncs === fail) throw refused("GeneralException");
      for (const act of batch) act(); // the first act that throws loses the rest of the batch, as in Office.js
    },
  };
  host.sandbox = {
    Excel: { run: (batch) => batch(context) },
    Office: { context: { diagnostics: { version: "fake build", platform: "fake" } } },
    console: { log: (line) => host.lines.push(JSON.parse(line)) },
  };
  return host;
}

async function run(options) {
  const host = makeHost(options);
  vm.createContext(host.sandbox);
  vm.runInContext(source, host.sandbox);
  await vm.runInContext("run()", host.sandbox);
  return host;
}

// ---- README.md's tables ------------------------------------------------------------------------------------------
// The rows of the table under a heading: the cells of each row whose first cell is in backticks.
function table(heading) {
  const start = readme.indexOf(heading);
  assert.ok(start >= 0, heading);
  const rows = [];
  for (const line of readme.slice(start).split("\n").slice(1)) {
    if (line.startsWith("#")) break;
    if (line.startsWith("| `")) rows.push(line.split("|").slice(1, -1).map((cell) => cell.trim().replace(/`/g, "")));
  }
  return rows;
}

// Scott's rule for README.md's four readings: the width, then each bin's upper edge and count from the minimum.
function scott(values, factor, sample) {
  const n = values.length;
  const mean = values.reduce((a, b) => a + b, 0) / n;
  const deviation = Math.sqrt(values.reduce((a, b) => a + (b - mean) ** 2, 0) / (sample ? n - 1 : n));
  const width = (factor * deviation) / Math.cbrt(n);
  const low = Math.min(...values);
  const bins = Math.ceil((Math.max(...values) - low) / width);
  const edges = Array.from({ length: bins }, (_, i) => low + (i + 1) * width);
  const counts = edges.map((edge, i) => values.filter((v) => v <= edge && (i === 0 || v > edges[i - 1])).length);
  return { width: width.toFixed(2), edges: edges.map((edge) => edge.toFixed(2)).join(", "), counts: counts.join(", ") };
}

async function main() {
  const charts = table("## The chart cases");
  const selections = table("## The twelve selections");
  const ids = [...charts, ...selections].map((row) => row[0]);
  assert.strictEqual(charts.length, 27, "the chart cases");
  assert.strictEqual(selections.length, 12, "the selections");

  // A fresh workbook.
  const fresh = await run();
  const order = fresh.lines.map((line) => line.case);
  assert.deepStrictEqual(order, ["ENV", ...ids, "END"], "one line a case, in README's order");
  assert.ok(fresh.lines.every((line) => line.probe === "chart-insert"));
  assert.deepStrictEqual(fresh.added, ids, "one sheet a case, named by the case");
  for (const [id, type, range] of charts) {
    const sheet = fresh.sheets.get(id);
    const line = fresh.lines.find((l) => l.case === id);
    assert.deepStrictEqual([...sheet.cells.keys()], [range], `${id}: its data are at README's range`);
    assert.strictEqual(sheet.charts.length, 1, `${id}: one chart`);
    const chart = sheet.charts[0];
    assert.deepStrictEqual(
      [chart.type, chart.range, chart.seriesBy, chart.name, chart.position],
      [type, range, "Auto", id, ["H2", "P20"]],
      `${id}: the chart README names, from the whole range, beside the data`,
    );
    const said = [line.type, line.range, line.made, line.read.chart];
    assert.deepStrictEqual(said, [type, range, true, { chartType: type }], id);
    const columns = sheet.cells.get(range)[0].length;
    assert.ok(columns <= 6, `${id}: the data end before column H, where the chart is`);
  }
  for (const [id, range] of selections) {
    const sheet = fresh.sheets.get(id);
    const holds = [[...sheet.cells.keys()], sheet.charts.length];
    assert.deepStrictEqual(holds, [[range], 0], `${id}: data at README's range, no chart`);
    const data = sheet.cells.get(range);
    const [, last] = range.split(":");
    assert.strictEqual(Number(/\d+$/.exec(last)[0]), data.length, `${id}: the range's rows`);
    assert.ok(data.every((row) => row.length === data[0].length), `${id}: a rectangle`);
  }
  const end = fresh.lines.at(-1);
  assert.deepStrictEqual([end.cases, end.made], [39, 27], "the END line counts the cases and the charts made");

  // The data README.md describes.
  // The data were made in the script's own realm: a copy through JSON compares as this realm's arrays do.
  const cells = (id) => JSON.parse(JSON.stringify([...fresh.sheets.get(id).cells.values()][0]));
  const numbers = (id) => cells(id).slice(1).map((row) => row[0]);
  assert.deepStrictEqual(numbers("Histogram1"), Array.from({ length: 20 }, (_, i) => i + 1));
  assert.deepStrictEqual(numbers("Histogram2"), [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
  assert.deepStrictEqual(cells("StockOHLC")[0], ["Date", "Open", "High", "Low", "Close"]);
  assert.deepStrictEqual(cells("StockVOHLC")[0], ["Date", "Volume", "Open", "High", "Low", "Close"]);
  for (const id of ["StockOHLC", "R08"]) {
    for (const [, open, high, low, close] of cells(id).slice(1)) {
      const ordered = low <= Math.min(open, close) && Math.max(open, close) <= high;
      assert.ok(ordered, `${id}: low, open, close, high in order`);
    }
    const formatted = [...fresh.sheets.get(id).formats.keys()];
    assert.deepStrictEqual(formatted, ["A2:A6"], `${id}: the dates have a date format`);
  }
  assert.strictEqual(cells("PieOfPie").length, 9, "eight items under a header");
  assert.strictEqual(cells("R06").slice(1).reduce((sum, row) => sum + row[1], 0), 1, "R06's shares sum to 1");
  const changes = cells("R07").slice(1).map((row) => row[1]);
  assert.ok(changes.some((v) => v < 0) && changes.some((v) => v > 0), "R07: mixed sign");
  const bare = cells("R04").flat().concat(cells("R10").flat());
  assert.ok(bare.every((v) => typeof v === "number"), "R04 and R10 have no header");
  assert.deepStrictEqual(cells("R03").slice(1, 3).map((row) => row[0]), [46023, 46054], "R03: 2026-01-01 and 2026-02-01");
  assert.strictEqual(cells("R12").length, 1000);

  // What each option set, and what was read back of it.
  const chartOf = (host, id) => host.sheets.get(id).charts[0];
  assert.deepStrictEqual(chartOf(fresh, "DataTable").table, { visible: true });
  assert.deepStrictEqual(chartOf(fresh, "Secondary").touched.at(-1), [1, { axisGroup: "Secondary" }]);
  assert.deepStrictEqual(chartOf(fresh, "Filtered").touched.at(-1), [1, { filtered: true }]);
  const lineOf = (host, id) => host.lines.find((line) => line.case === id);
  assert.strictEqual(lineOf(fresh, "Secondary").read.series2.axisGroup, "Secondary");
  assert.strictEqual(lineOf(fresh, "Filtered").read.series2.filtered, true);
  assert.strictEqual(lineOf(fresh, "DataTable").read.dataTable.visible, true);
  assert.deepStrictEqual(Object.keys(lineOf(fresh, "Histogram1").read.bins), [
    "type", "width", "count", "allowOverflow", "allowUnderflow", "overflowValue", "underflowValue",
  ]);
  assert.deepStrictEqual(Object.keys(lineOf(fresh, "PieOfPie").read.series1), ["splitType", "splitValue", "secondPlotSize"]);
  assert.deepStrictEqual(Object.keys(lineOf(fresh, "Bubble").read.series1), ["bubbleScale"]);

  // A host that refuses the 3-D types: each is its case's answer, and every other case is as in the fresh run.
  const flat = charts.map((row) => row[1]).filter((type) => type.startsWith("3D"));
  assert.strictEqual(flat.length, 6, "the 3-D cases");
  const twoD = await run({ refuse: flat });
  for (const line of twoD.lines.slice(1, -1)) {
    const same = fresh.lines.find((l) => l.case === line.case);
    if (flat.includes(line.type)) {
      const answer = [line.made, line.error, twoD.sheets.get(line.case).charts.length];
      assert.deepStrictEqual(answer, [false, "InvalidArgument", 0], line.case);
      assert.strictEqual(twoD.sheets.get(line.case).cells.size, 1, `${line.case}: its data are still written`);
    } else {
      assert.deepStrictEqual(line, same, `${line.case}: as in the fresh run`);
    }
  }
  assert.strictEqual(twoD.lines.at(-1).made, 21);

  // A host with no data table, and one with no bin options.
  const noTable = await run({ lacks: ["dataTable"] });
  assert.deepStrictEqual(lineOf(noTable, "DataTable").read.dataTable, { isNullObject: true });
  const noBins = await run({ lacks: ["binOptions"] });
  assert.strictEqual(lineOf(noBins, "Histogram1").read.bins.error, "ApiNotFound");
  for (const [host, changed] of [[noTable, ["DataTable"]], [noBins, ["Histogram1", "Histogram2"]]]) {
    for (const line of host.lines) {
      if (!changed.includes(line.case)) assert.deepStrictEqual(line, lineOf(fresh, line.case), line.case);
    }
    assert.strictEqual(host.lines.at(-1).made, 27);
  }

  // A workbook that already has one of the sheets: one line, nothing written.
  const used = await run({ existing: ["Sheet1", "Radar"] });
  assert.deepStrictEqual(used.lines.map((line) => line.case), ["ENV"]);
  assert.ok(used.lines[0].refused.includes("Radar"));
  assert.deepStrictEqual([used.added, used.charted, used.sheets.get("Radar").cells.size], [[], [], 0]);

  // Any one sync may fail: no exception, nothing added twice, and at most one case differs from the fresh run.
  for (let fail = 1; fail <= fresh.syncs; fail++) {
    const partial = await run({ fail });
    assert.strictEqual(partial.lines[0].case, "ENV", `sync ${fail}`);
    assert.strictEqual(new Set(partial.added).size, partial.added.length, `sync ${fail}: no sheet twice`);
    assert.strictEqual(new Set(partial.charted).size, partial.charted.length, `sync ${fail}: no chart twice`);
    if (fail === 1) {
      assert.deepStrictEqual([partial.lines.length, partial.added.length], [1, 0], "the first sync failing writes nothing");
      assert.ok(partial.lines[0].refused);
      continue;
    }
    const cases = partial.lines.map((line) => line.case);
    assert.deepStrictEqual(cases, ["ENV", ...ids, "END"], `sync ${fail}: every case has a line`);
    const differing = partial.lines.slice(1, -1).filter((line) => {
      const same = fresh.lines.find((l) => l.case === line.case);
      return JSON.stringify(line) !== JSON.stringify(same);
    });
    assert.strictEqual(differing.length, 1, `sync ${fail}: one case tells of the failure`);
    assert.ok(JSON.stringify(differing[0]).includes("GeneralException"), `sync ${fail}: with the error's code`);
    // The lines say which charts the workbook holds, whatever failed after a chart was made.
    for (const [id] of charts) {
      const held = (partial.sheets.get(id) || { charts: [] }).charts.length === 1;
      assert.strictEqual(lineOf(partial, id).made === true, held, `sync ${fail}: ${id}'s line and the workbook agree`);
    }
    const madeLines = partial.lines.filter((line) => line.made === true).length;
    assert.strictEqual(partial.lines.at(-1).made, madeLines, `sync ${fail}: the END line counts them`);
  }

  // Scott's rule, for README.md's table of readings.
  const scottRows = table("## What Scott's rule gives");
  const data = { Histogram1: numbers("Histogram1"), Histogram2: numbers("Histogram2") };
  assert.strictEqual(scottRows.length, 8, "four readings of two data sets");
  for (const [id, factor, deviation, width, edges, counts] of scottRows) {
    const want = scott(data[id], Number(factor), deviation === "sample");
    assert.deepStrictEqual([width, edges, counts], [want.width, want.edges, want.counts], `${id} ${factor} ${deviation}`);
  }
  const counted = new Set(scottRows.filter((row) => row[0] === "Histogram1").map((row) => `${row[2]}: ${row[5]}`));
  assert.strictEqual(counted.size, 2, "Histogram1's counts tell the sample deviation from the population's");

  // In Script Lab: the one button runs the script and is then off, and its lines are shown under it.
  const open = () => {
    const page = fakePage();
    const host = makeHost();
    host.sandbox.document = page.document;
    vm.createContext(host.sandbox);
    vm.runInContext(source, host.sandbox);
    return { page, logged: () => host.lines.map((line) => JSON.stringify(line)) };
  };
  await checkButtons(open, ["RUN"], (line) => (line.probe === "chart-insert" ? "RUN" : line.probe));
  console.log(
    `ok: ${fresh.lines.length} lines, ${fresh.added.length} sheets and ${fresh.charted.length} charts in a fresh workbook; ` +
      `${flat.length} refused types answered; ${fresh.syncs} syncs failed in turn; a button`,
  );
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
