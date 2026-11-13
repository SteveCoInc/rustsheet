// Probe `chart-series-name` (README.md beside this file). Office.js, in Excel for the web.
//
// In a workbook opened from one of the generated files:
//   READ    what the chart plots and what Office.js says its series read; then download a copy
//   CHANGE  a cell under the name changes, the name is redefined, the name is deleted; then download a copy
// In a second, fresh copy of series-control.xlsx:
//   SET     a series is given a name through the API; then download a copy
//
// Each run logs one JSON line. A call Excel refuses is logged with its error where its answer would be, and
// the run goes on: a refusal is an answer.
// Usage: by the case's button in Script Lab (the end of this file), or by run("READ").

const DIMENSIONS = ["Categories", "Values"];
const SETS = ["1.7", "1.12", "1.15", "1.21"];
// SET: the strings the series' values are set to, in this order. Office.js takes a string since ExcelApi 1.21
// ("a cell or table reference, a defined name, or a literal value or array"). The last one is the spelling a
// formula uses, so that the copy downloaded afterwards holds it if Excel took it.
const SPELLINGS = ["Sales", "=[0]!Sales", "=Sheet1!Sales", "=Sales"];

/** Runs `work` and files its answer, or the error it threw, under `key`. */
async function attempt(out, key, work) {
  try {
    out[key] = await work();
  } catch (error) {
    const where = error && error.debugInfo ? error.debugInfo.errorLocation : undefined;
    out[key] = { error: (error && error.code) || String(error), message: error && error.message, where };
  }
}

// Every call asks for its objects again: one that was queued in a request Excel refused is not used twice.
function sheet1(context) {
  return context.workbook.worksheets.getItem("Sheet1");
}

/** The one series of the one chart of Sheet1. */
function theSeries(context) {
  return sheet1(context).charts.getItemAt(0).series.getItemAt(0);
}

/** The series' name, and for each dimension the values plotted, the source as text and the source's kind. */
async function readSeries(context) {
  const out = {};
  await attempt(out, "name", async () => {
    const series = theSeries(context);
    series.load("name");
    await context.sync();
    return series.name;
  });
  for (const dimension of DIMENSIONS) {
    const calls = {
      values: (series) => series.getDimensionValues(dimension),
      source: (series) => series.getDimensionDataSourceString(dimension),
      sourceType: (series) => series.getDimensionDataSourceType(dimension),
    };
    for (const [what, call] of Object.entries(calls)) {
      await attempt(out, `${what}:${dimension}`, async () => {
        const result = call(theSeries(context));
        await context.sync();
        return result.value;
      });
    }
  }
  return out;
}

/** The workbook's names and Sheet1's own, each with its scope and formula. */
async function readNames(context) {
  const out = {};
  const collections = {
    workbook: () => context.workbook.names,
    sheet: () => sheet1(context).names,
  };
  for (const [scope, collection] of Object.entries(collections)) {
    await attempt(out, scope, async () => {
      const names = collection();
      names.load("items/name,items/scope,items/formula");
      await context.sync();
      return names.items.map((item) => ({ name: item.name, scope: item.scope, formula: item.formula }));
    });
  }
  return out;
}

/** The name `Sales`, in the workbook's names or, when it is not there, in Sheet1's; null when in neither. */
async function salesName(context) {
  for (const names of [() => context.workbook.names, () => sheet1(context).names]) {
    const item = names().getItemOrNullObject("Sales");
    await context.sync();
    if (!item.isNullObject) return item;
  }
  return null;
}

/** One step of a case: `change` is queued and synced, then the names and the series are read. */
async function step(context, out, key, change) {
  const result = {};
  await attempt(result, "change", async () => {
    const done = await change();
    await context.sync();
    return done === undefined ? "done" : done;
  });
  result.names = await readNames(context);
  result.series = await readSeries(context);
  out[key] = result;
}

/** C2:C5, the cells no name names: a series set to them reads 1, 2, 3, 4. */
function otherCells(context) {
  return sheet1(context).getRange("C2:C5");
}

const CASES = {
  // What Excel made of the file: nothing is changed but the marker in E1, which makes Excel save the workbook.
  READ: async (context, out) => {
    await attempt(out, "cells", async () => {
      const range = sheet1(context).getRange("A1:C5");
      range.load("values");
      await context.sync();
      return range.values;
    });
    out.names = await readNames(context);
    out.series = await readSeries(context);
    await attempt(out, "marker", async () => {
      sheet1(context).getRange("E1").values = [["read"]];
      await context.sync();
      return "written";
    });
  },
  // Does the chart follow the cells under the name, the name when it is redefined, and what is left of it
  // when the name is deleted?
  CHANGE: async (context, out) => {
    await step(context, out, "cell", async () => {
      sheet1(context).getRange("B2").values = [[11]];
    });
    await step(context, out, "redefine", async () => {
      const name = await salesName(context);
      if (name === null) return "no name Sales";
      name.formula = "=Sheet1!$C$2:$C$5";
      return undefined;
    });
    await step(context, out, "delete", async () => {
      const name = await salesName(context);
      if (name === null) return "no name Sales";
      name.delete();
      return undefined;
    });
  },
  // Can a series be given a name through the API, and what does Excel then say it reads? Before each attempt
  // the series is set to C2:C5, so that an attempt Excel takes reads 10, 20, 15, 25 and one it ignores or
  // refuses still reads 1, 2, 3, 4. It runs only where the workbook's `Sales` is B2:B5 and B2 is 10, as in a
  // fresh copy of the control: not where CHANGE has run, and not where the names are the sheet's.
  SET: async (context, out) => {
    const fresh = {};
    await attempt(fresh, "is", async () => {
      const name = context.workbook.names.getItemOrNullObject("Sales");
      name.load("formula");
      const cell = sheet1(context).getRange("B2");
      cell.load("values");
      await context.sync();
      return !name.isNullObject && name.formula === "=Sheet1!$B$2:$B$5" && cell.values[0][0] === 10;
    });
    if (fresh.is !== true) {
      out.error = "not a fresh series-control.xlsx: nothing changed";
      out.fresh = fresh.is;
      return;
    }
    const toOther = async () => theSeries(context).setValues(otherCells(context));
    const reset = (key) => step(context, out, `reset before ${key}`, toOther);
    await reset("range of Sales");
    await step(context, out, "range of Sales", async () => {
      theSeries(context).setValues(context.workbook.names.getItem("Sales").getRange());
    });
    for (const spelling of SPELLINGS) {
      await reset(`values ${spelling}`);
      await step(context, out, `values ${spelling}`, async () => {
        theSeries(context).setValues(spelling);
      });
    }
    await step(context, out, "reset before categories =Months", async () => {
      theSeries(context).setXAxisValues(otherCells(context));
    });
    await step(context, out, "categories =Months", async () => {
      theSeries(context).setXAxisValues("=Months");
    });
  },
};

/** What the workbook holds where the probe's chart should be: Sheet1, its charts, the first chart's series. */
async function found(context) {
  const out = {};
  await attempt(out, "sheet", async () => {
    const sheet = context.workbook.worksheets.getItemOrNullObject("Sheet1");
    await context.sync();
    return !sheet.isNullObject;
  });
  if (out.sheet !== true) return out;
  await attempt(out, "charts", async () => {
    const count = sheet1(context).charts.getCount();
    await context.sync();
    return count.value;
  });
  if (out.charts !== 1) return out;
  await attempt(out, "series", async () => {
    const count = sheet1(context).charts.getItemAt(0).series.getCount();
    await context.sync();
    return count.value;
  });
  return out;
}

async function run(id) {
  const out = { probe: "chart-series-name", case: id };
  try {
    if (!Object.prototype.hasOwnProperty.call(CASES, id)) {
      out.error = "unknown case";
      return out;
    }
    await Excel.run(async (context) => {
      // On a probe file, a chart or a series that is not there is an answer: Excel dropped it.
      out.workbook = await found(context);
      if (out.workbook.sheet !== true || out.workbook.charts !== 1 || out.workbook.series !== 1) {
        out.error = "no Sheet1 with one chart of one series: nothing more read, nothing changed";
        return;
      }
      await attempt(out, "file", async () => {
        context.workbook.load("name");
        await context.sync();
        return context.workbook.name;
      });
      await attempt(out, "env", async () => ({
        platform: String(Office.context.platform),
        version: Office.context.diagnostics ? Office.context.diagnostics.version : null,
        sets: Object.fromEntries(SETS.map((set) => [set, Office.context.requirements.isSetSupported("ExcelApi", set)])),
      }));
      await CASES[id](context, out);
    });
  } catch (error) {
    out.failed = { error: (error && error.code) || String(error), message: error && error.message };
  } finally {
    console.log(JSON.stringify(out));
  }
  return out;
}

// ---- In Script Lab ---------------------------------------------------------------------------------------------
// This file is the Script tab, and the HTML tab holds one line: <div id="cases"></div><pre id="out"></pre>
// Press Run, then click a case's button. Its JSON line is logged, and shown under the buttons to copy from. The
// cases run once each in a workbook, in this order: while a case runs every button is off, and afterwards its
// own button and the ones before it stay off.
if (typeof document !== "undefined" && document.getElementById("cases")) {
  const print = console.log;
  console.log = (line) => {
    print(line);
    document.getElementById("out").textContent += `${line}\n`;
  };
  const buttons = [];
  const spent = new Set(); // the buttons whose case has run, and the ones before them
  for (const id of Object.keys(CASES)) {
    const button = document.createElement("button");
    button.textContent = id;
    button.onclick = async () => {
      buttons.slice(0, buttons.indexOf(button) + 1).forEach((done) => spent.add(done));
      buttons.forEach((other) => (other.disabled = true));
      try {
        await run(id);
      } finally {
        buttons.forEach((other) => (other.disabled = spent.has(other)));
      }
    };
    buttons.push(button);
    document.getElementById("cases").appendChild(button);
  }
}
