// Office.js (Script Lab), in Excel for the web, the set of record (README.md, "How"). Two cases in one new, empty
// workbook, with a person's five moves between them:
//   SETUP adds five sheets, one for each move of README.md. On each, rows 10 to 14 are a block that is unlocked,
//         formula-hidden, an allow-edit range ("Grant") and the range a SUM reads;
//   READ  reads, from each of the five sheets, every row's label and its two format flags, the allow-edit ranges
//         and the SUM's formula, after the moves.
// Each logs one JSON line. Requirement sets: ExcelApi 1.2 (format.protection), ExcelApiOnline 1.1
// (protection.allowEditRanges), which Excel for the web has.
//
// Usage: their buttons in Script Lab (the end of this file), SETUP and then READ, or run("SETUP") and run("READ").

const log = (line) => console.log(JSON.stringify(line));

const MOVES = ["INNER", "WHOLE", "FIRST", "LAST", "INTO"]; // the sheets' names: README.md says the move of each
const ROWS = 30; // rows that hold a label
const READ_ROWS = 36; // rows READ looks at: a block moved down may end past row 30
const BLOCK = "A10:C14";

// What one queued step gave after its own sync, or the error's code: a host without allow-edit ranges still
// answers for the flags and the SUM.
async function attempt(context, step) {
  try {
    const read = step();
    await context.sync();
    return { ok: typeof read === "function" ? read() : true };
  } catch (e) {
    return { error: e.code || String(e), message: e.message, at: e.debugInfo && e.debugInfo.errorLocation };
  }
}

const CASES = {
  SETUP: async (context) => {
    const sheets = context.workbook.worksheets;
    const made = MOVES.map((name) => {
      const sheet = sheets.add(name);
      // A row's label is the row it started in, so that READ tells where each row went.
      sheet.getRange(`A1:A${ROWS}`).values = Array.from({ length: ROWS }, (_, i) => [`r${i + 1}`]);
      sheet.getRange(`B1:B${ROWS}`).values = Array.from({ length: ROWS }, () => [1]);
      const block = sheet.getRange(BLOCK);
      block.format.protection.locked = false;
      block.format.protection.formulaHidden = true;
      // The control: how a plain reference to the block's rows follows the same move.
      sheet.getRange("E1").formulas = [["=SUM(B10:B14)"]];
      return sheet;
    });
    await context.sync();
    const grants = {};
    for (const sheet of made) {
      const added = await attempt(context, () => sheet.protection.allowEditRanges.add("Grant", BLOCK));
      grants[MOVES[made.indexOf(sheet)]] = added;
    }
    return { sheets: MOVES, block: BLOCK, grants };
  },
  READ: async (context) => {
    const out = {};
    for (const name of MOVES) {
      const sheet = context.workbook.worksheets.getItem(name);
      const labels = sheet.getRange(`A1:A${READ_ROWS}`);
      labels.load("values");
      const flags = Array.from({ length: READ_ROWS }, (_, i) => sheet.getRange(`A${i + 1}`).format.protection);
      flags.forEach((f) => f.load("locked,formulaHidden"));
      const sum = sheet.getRange("E1");
      sum.load("formulas,values");
      await context.sync();
      const ranges = sheet.protection.allowEditRanges;
      const grants = await attempt(context, () => {
        ranges.load("items/title,items/address");
        return () => ranges.items.map((g) => [g.title, g.address]);
      });
      out[name] = {
        // [the row now, its label, locked, formulaHidden]: only rows that hold a label or differ from a plain row
        rows: flags
          .map((f, i) => [i + 1, labels.values[i][0], f.locked, f.formulaHidden])
          .filter(([, label, locked, hidden]) => label !== "" || locked !== true || hidden !== false),
        grants: "ok" in grants ? grants.ok : grants,
        sum: [sum.formulas[0][0], sum.values[0][0]],
      };
    }
    return out;
  },
};

async function run(id) {
  await Excel.run(async (context) => {
    const env = { build: Office.context.diagnostics.version, platform: Office.context.diagnostics.platform };
    try {
      if (!CASES[id]) throw { code: "UnknownCase", message: `no case ${id}` };
      const sheets = context.workbook.worksheets;
      sheets.load("items/name");
      await context.sync();
      const names = sheets.items.map((sheet) => sheet.name);
      const made = MOVES.every((name) => names.includes(name));
      if (id === "SETUP") {
        // A new, empty workbook only.
        const used = sheets.items.map((sheet) => sheet.getUsedRangeOrNullObject(true));
        used.forEach((u) => u.load("address"));
        await context.sync();
        const full = used.filter((u) => !u.isNullObject).map((u) => u.address);
        if (full.length || made) {
          throw { code: "WorkbookNotEmpty", message: `run SETUP in a new, empty workbook: ${names.join(", ")}` };
        }
      } else if (!made) {
        throw { code: "NotSetUp", message: `run SETUP first: the sheets are ${names.join(", ")}` };
      }
      log({ case: id, ...env, data: await CASES[id](context) });
    } catch (e) {
      log({ case: id, ...env, error: e.code || String(e), message: e.message });
    }
  });
}

// ---- In Script Lab ---------------------------------------------------------------------------------------------
// This file is the Script tab, and the HTML tab holds one line: <div id="cases"></div><pre id="out"></pre>
// Press Run, click SETUP, make the five moves of README.md, then click READ. While a case runs both buttons are
// off; afterwards its own button and the one before it stay off. Each JSON line is logged, and shown under the
// buttons to copy from.
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
