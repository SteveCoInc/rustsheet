// Probe request `chart-insert`: what Excel for the web makes when a chart is inserted (README.md).
// Office.js (Script Lab), in Excel for the web. Run it once, in a FRESH workbook, by its button in Script Lab
// (the end of this file) or by await run();
// It adds one sheet a case, named by the case, writes the case's data to it and, for a chart case, adds the chart.
// It logs one JSON line a case, with an ENV line before them and an END line after. The saved workbook is the
// answer: the lines say which charts the host made and what the API reads back of them.
// A workbook that already has a sheet named as a case is refused with one line, and nothing is written to it.
// Requirement sets: ExcelApi 1.1 (worksheets.add, charts.add, setPosition), 1.7 (chartType, filtered, gapWidth),
// 1.8 (axisGroup, overlap, splitType, secondPlotSize), 1.9 (binOptions, bubbleScale, splitValue), 1.14 (the data
// table).
// A call the host lacks, or a chart the host refuses, is logged with its error code in place of its answer.

const log = (line) => console.log(JSON.stringify({ probe: "chart-insert", ...line }));
const code = (e) => ({ error: e.code || String(e), message: e.message });

// ---- The data --------------------------------------------------------------------------------------------------
// A1:D6: series names in row 1, text categories in column A (the charts lane's P-CH-8 data).
const REGIONS = [
  ["Region", "Alpha", "Beta", "Gamma"],
  ["North", 12, 30, 7],
  ["South", 19, 24, 11],
  ["East", 8, 35, 16],
  ["West", 23, 18, 5],
  ["Centre", 15, 27, 9],
];
const TWO_SERIES = REGIONS.map((row) => row.slice(0, 3));
const POINTS = [["X", "Y", "Size"], [1, 10, 4], [2, 14, 9], [3, 9, 2], [4, 18, 12], [5, 12, 6]];
// Five trading days from Thursday 2026-01-01, as serial numbers; shown as dates by the number format.
const DAYS = [46023, 46024, 46027, 46028, 46029];
const PRICES = {
  Volume: [1500, 1800, 1200, 2100, 1700],
  Open: [20, 22, 21, 25, 24],
  High: [24, 25, 26, 27, 28],
  Low: [18, 20, 19, 23, 22],
  Close: [22, 21, 25, 24, 27],
};
const stock = (...names) => [["Date", ...names], ...DAYS.map((day, i) => [day, ...names.map((name) => PRICES[name][i])])];
const SHARES = [["Item", "Share"], ["A", 30], ["B", 22], ["C", 15], ["D", 12], ["E", 9], ["F", 6], ["G", 4], ["H", 2]];
const column = (header, values) => [...header, ...values].map((value) => [value]);
const ONE_TO_TWENTY = column(["Values"], Array.from({ length: 20 }, (_, i) => i + 1));
const FIBONACCI = column(["Values"], [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
// The first of each month of 2026, as serial numbers.
const MONTHS = Array.from({ length: 12 }, (_, month) => Date.UTC(2026, month, 1) / 86400000 + 25569);
const MONTH_NAMES = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const SALES = [12, 19, 8, 23, 15, 27, 31, 22, 17, 29, 35, 26];

// ---- The cases -------------------------------------------------------------------------------------------------
// A chart case: `type` is the Excel.ChartType string, `data` goes to A1 and is the chart's source, `dates` is the
// number format of A2 down, `option` is set after the chart is made, `series` names what is read of series 1.
const typed = (types, data, more = {}) => types.map((type) => ({ id: type, type, data, ...more }));
const CASES = [
  ...typed(["Radar", "RadarMarkers", "RadarFilled"], REGIONS),
  ...typed(["Bubble", "Bubble3DEffect"], POINTS, { series: "bubbleScale" }),
  { id: "StockHLC", type: "StockHLC", data: stock("High", "Low", "Close"), dates: "d-mmm" },
  { id: "StockOHLC", type: "StockOHLC", data: stock("Open", "High", "Low", "Close"), dates: "d-mmm" },
  { id: "StockVHLC", type: "StockVHLC", data: stock("Volume", "High", "Low", "Close"), dates: "d-mmm" },
  { id: "StockVOHLC", type: "StockVOHLC", data: stock("Volume", "Open", "High", "Low", "Close"), dates: "d-mmm" },
  ...typed(["Surface", "SurfaceWireframe", "SurfaceTopView", "SurfaceTopViewWireframe"], REGIONS),
  ...typed(["PieOfPie", "BarOfPie"], SHARES, { series: "splitType,splitValue,secondPlotSize" }),
  ...typed(["3DColumnClustered", "3DColumn", "3DBarClustered", "3DLine", "3DArea"], REGIONS),
  { id: "3DPie", type: "3DPie", data: SHARES },
  { id: "Histogram1", type: "Histogram", data: ONE_TO_TWENTY, option: "bins" },
  { id: "Histogram2", type: "Histogram", data: FIBONACCI, option: "bins" },
  { id: "DataTable", type: "ColumnClustered", data: REGIONS, option: "dataTable", series: "gapWidth,overlap" },
  { id: "Secondary", type: "ColumnClustered", data: TWO_SERIES, option: "secondary" },
  { id: "Filtered", type: "ColumnClustered", data: REGIONS, option: "filtered" },
  { id: "Lines", type: "Line", data: TWO_SERIES },
  // The twelve selections of the charts lane's CH-REC-1: data only, for Recommended Charts by hand.
  { id: "R01", data: REGIONS.slice(0, 5).map((row) => row.slice(0, 2)) },
  { id: "R02", data: REGIONS.slice(0, 5) },
  { id: "R03", data: [["Month", "Sales"], ...MONTHS.map((month, i) => [month, SALES[i]])], dates: "mmm-yy" },
  { id: "R04", data: POINTS.slice(1).map((row) => row.slice(0, 2)) },
  { id: "R05", data: POINTS.slice(1, 6) },
  { id: "R06", data: [["Item", "Share"], ["A", 0.5], ["B", 0.3], ["C", 0.2]] },
  { id: "R07", data: [["Item", "Change"], ...[7, -3, 12, -8, 5, -1, 9, -6, 4, -2].map((v, i) => [`Item ${i + 1}`, v])] },
  { id: "R08", data: stock("Open", "High", "Low", "Close"), dates: "d-mmm" },
  {
    id: "R09",
    data: [
      ["X", "Y", "Group"],
      ...Array.from({ length: 19 }, (_, i) => [i + 1, ((i * 7) % 11) + 3, ["Red", "Green", "Blue"][i % 3]]),
    ],
  },
  { id: "R10", data: column([], [4, 9, 2, 12, 6, 15, 3, 8, 11, 7]) },
  { id: "R11", data: [["", ...MONTH_NAMES], ["Sales", ...SALES]] },
  { id: "R12", data: [["Index", "Value"], ...Array.from({ length: 999 }, (_, i) => [i + 1, ((i + 1) * 37) % 101])] },
];

// What a case sets on its chart after the host made it, and what it then reads back: [label, object, properties].
const OPTIONS = {
  // The automatic bins of a histogram: nothing is set; the series' bin options are read.
  bins: (chart) => {
    const names = "type,width,count,allowOverflow,allowUnderflow,overflowValue,underflowValue";
    return [["bins", chart.series.getItemAt(0).binOptions, names]];
  },
  dataTable: (chart) => {
    const table = chart.getDataTableOrNullObject();
    table.visible = true;
    return [["dataTable", table, "visible,showLegendKey,showHorizontalBorder,showVerticalBorder,showOutlineBorder"]];
  },
  secondary: (chart) => {
    const series = chart.series.getItemAt(1);
    series.axisGroup = "Secondary";
    return [["series2", series, "axisGroup"]];
  },
  filtered: (chart) => {
    const series = chart.series.getItemAt(1);
    series.filtered = true;
    return [["series2", series, "filtered"]];
  },
};

const letters = (n) => (n > 26 ? letters(Math.floor((n - 1) / 26)) : "") + String.fromCharCode(65 + ((n - 1) % 26));
const address = (data) => `A1:${letters(data[0].length)}${data.length}`;

// Loads `names` of one object and gives them, or the error's code: a property the host lacks is an answer.
async function read(context, object, names) {
  try {
    object.load(names);
    await context.sync();
    if (object.isNullObject) return { isNullObject: true };
    return Object.fromEntries(names.split(",").map((name) => [name, object[name]]));
  } catch (e) {
    return code(e);
  }
}

// One case: its sheet and data, then its chart, then its option, then what is read back. Each step that fails
// is logged in its place, and the steps after it in the case are not run.
async function one(context, c) {
  const result = { range: address(c.data) };
  let sheet;
  try {
    sheet = context.workbook.worksheets.add(c.id);
    sheet.getRange(result.range).values = c.data;
    if (c.dates) sheet.getRange(`A2:A${c.data.length}`).numberFormat = c.data.slice(1).map(() => [c.dates]);
    await context.sync();
  } catch (e) {
    return { ...result, sheet: code(e) };
  }
  if (!c.type) return result;
  result.type = c.type;
  let chart;
  try {
    chart = sheet.charts.add(c.type, sheet.getRange(result.range), "Auto");
    chart.name = c.id;
    chart.setPosition("H2", "P20");
    await context.sync();
    result.made = true;
  } catch (e) {
    return { ...result, made: false, ...code(e) };
  }
  result.read = { chart: await read(context, chart, "chartType") };
  if (c.series) result.read.series1 = await read(context, chart.series.getItemAt(0), c.series);
  if (c.option) {
    let reads = [];
    try {
      reads = OPTIONS[c.option](chart);
      await context.sync();
      result.option = "set";
    } catch (e) {
      result.option = code(e);
    }
    for (const [label, object, names] of reads) result.read[label] = await read(context, object, names);
  }
  return result;
}

async function run() {
  await Excel.run(async (context) => {
    let env = {};
    try {
      env = { build: Office.context.diagnostics.version, platform: Office.context.diagnostics.platform };
    } catch (e) {
      env = code(e);
    }
    let taken;
    try {
      const sheets = context.workbook.worksheets;
      sheets.load("items/name");
      await context.sync();
      taken = sheets.items.map((sheet) => sheet.name).filter((name) => CASES.some((c) => c.id === name));
    } catch (e) {
      // The sheets could not be listed, so nothing says the workbook is fresh: nothing is written.
      log({ case: "ENV", env, refused: "the workbook's sheets could not be listed", ...code(e) });
      return;
    }
    if (taken.length > 0) {
      log({ case: "ENV", env, refused: `this workbook already has ${taken.join(", ")}: run in a fresh workbook` });
      return;
    }
    log({ case: "ENV", env });
    let made = 0;
    for (const c of CASES) {
      let result;
      try {
        result = await one(context, c);
      } catch (e) {
        result = code(e);
      }
      made += result.made ? 1 : 0;
      log({ case: c.id, ...result });
    }
    log({ case: "END", cases: CASES.length, made });
  });
}

// ---- In Script Lab ---------------------------------------------------------------------------------------------
// This file is the Script tab, and the HTML tab holds one line: <div id="cases"></div><pre id="out"></pre>
// Press Run, then click RUN: the click turns the button off, since a run needs a fresh workbook. Its JSON lines
// are logged, and shown under the button to copy from.
if (typeof document !== "undefined" && document.getElementById("cases")) {
  const print = console.log;
  console.log = (line) => {
    print(line);
    document.getElementById("out").textContent += `${line}\n`;
  };
  const button = document.createElement("button");
  button.textContent = "RUN";
  button.onclick = () => {
    button.disabled = true;
    return run();
  };
  document.getElementById("cases").appendChild(button);
}
