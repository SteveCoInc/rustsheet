// Office.js (Script Lab), in Excel for the web, the set of record (README.md, "How"). Two cases, each in a new,
// empty workbook of its own: NAMES asks for every candidate but one, and NUL for the name that holds U+0000,
// so that a host that stops at it does not take the others' workbook with it. For each candidate sheet name a
// case asks Excel twice, by adding a sheet under the name (and deleting it) and by renaming a sheet to it,
// reads the name back, reads how Excel spells a reference to the sheet, and enters that spelling as a formula on
// the first sheet. It logs one JSON line a candidate and a last line with the workbook's sheet names; every
// line says its `case`. A name is logged as its UTF-16 code units: the characters asked about do not survive a
// copy from a console.
// Requirement set: ExcelApi 1.4 (Worksheet.getUsedRangeOrNullObject, for the empty-workbook check; the rest is
// 1.1: worksheets.add, Worksheet.name and delete, Range.address, formulas, values).
//
// Usage: a case's button in Script Lab (the end of this file), or run("NAMES") and run("NUL").

const log = (line) => console.log(JSON.stringify(line));
const units = (text) => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));

// The candidates of each case: an id, and the name asked for. README.md says why each is asked. No two names
// hold the same letters, so that no two become one name on a host that drops a character, puts another in its
// place, trims, or stops at U+0000: a name another sheet holds is refused for that alone, whatever its characters.
const CANDIDATES = {
  NAMES: [
    ["plain", "Plain"],
    ["tab", "a\tb"],
    ["line-feed", "c\nd"],
    ["u0001", "e\u0001f"],
    ["u001f", "g\u001Fh"],
    ["u007f", "i\u007Fj"],
    ["spaces-only", "   "],
    ["tab-only", "\t"],
    ["nbsp-only", "\u00A0"],
    ["outer-spaces", " k "],
    ["astral-16", "\uD83D\uDE00".repeat(16)], // 16 characters outside the BMP: 32 UTF-16 units
    ["apostrophe-end", "o'"],
    ["history", "History"],
  ],
  NUL: [["u0000", "m\u0000n"]],
};

// What one queued step gave: its value, or the error's code, message and place (Office.js names the statement
// that failed, where a step queues more than one). A failed sync drops what was queued with it.
async function attempt(context, step) {
  try {
    const read = step();
    await context.sync();
    return { ok: typeof read === "function" ? read() : true };
  } catch (e) {
    return { error: e.code || String(e), message: e.message, at: e.debugInfo && e.debugInfo.errorLocation };
  }
}

// One candidate, on row `row` of the first sheet. The sheet a candidate ends on holds the candidate's id in C1,
// so that the downloaded file says which sheet is which, whatever its name became.
async function candidate(context, home, row, which, id, name) {
  const out = { case: which, candidate: id, asked: units(name) };
  const sheets = context.workbook.worksheets;
  // Asked by adding a sheet under the name. The sheet is deleted again, so that the second way can ask for the
  // same name.
  /** @type {Excel.Worksheet} */
  let added;
  out.added = await attempt(context, () => {
    added = sheets.add(name);
    added.load("name");
    return () => units(added.name);
  });
  if ("ok" in out.added) out.addedDeleted = await attempt(context, () => added.delete());
  // Asked by renaming a sheet, as the rename box does.
  /** @type {Excel.Worksheet} */
  let sheet;
  const fresh = await attempt(context, () => {
    sheet = sheets.add();
    sheet.load("name");
    return () => units(sheet.name);
  });
  if ("error" in fresh) {
    out.fresh = fresh; // no sheet to rename: the name was not asked for this way
    return out;
  }
  out.renamed = await attempt(context, () => {
    sheet.name = name;
    sheet.load("name");
    return () => units(sheet.name);
  });
  out.sheet = "renamed";
  if ("error" in out.renamed && "ok" in out.added) {
    // Only adding takes the name: the candidate's sheet is one added under it once more.
    const again = await attempt(context, () => {
      sheet.delete();
      sheet = sheets.add(name);
      sheet.load("name");
      return () => units(sheet.name);
    });
    if ("error" in again) {
      out.again = again;
      return out;
    }
    out.sheet = "added again";
  } else if ("error" in out.renamed) {
    out.sheet = "not named"; // neither way took the name: the sheet keeps the name Excel gave it
  }
  // How Excel spells a reference to the candidate's sheet, and that spelling entered as a formula.
  /** @type {Excel.Range} */
  let cell;
  const address = await attempt(context, () => {
    sheet.getRange("C1").values = [[id]];
    cell = sheet.getRange("A1");
    cell.values = [[row]];
    cell.load("address");
    return () => cell.address;
  });
  if ("error" in address) {
    out.address = address;
    return out;
  }
  out.address = { ok: units(address.ok) };
  /** @type {Excel.Range} */
  let reader;
  out.reader = await attempt(context, () => {
    reader = home.getRange(`B${row}`);
    reader.formulas = [[`=${address.ok}`]];
    reader.load("formulas,values");
    return () => ({ formula: units(reader.formulas[0][0]), value: reader.values[0][0] });
  });
  return out;
}

// A case: its candidates in turn, each on its own row of the first sheet, then the workbook's sheet names.
const asked = (which) => async (context, home) => {
  const list = CANDIDATES[which];
  for (let i = 0; i < list.length; i++) {
    const [id, name] = list[i];
    log(await candidate(context, home, i + 1, which, id, name));
  }
  const sheets = context.workbook.worksheets;
  sheets.load("items/name");
  await context.sync();
  return { sheets: sheets.items.map((sheet) => units(sheet.name)) };
};
const CASES = { NAMES: asked("NAMES"), NUL: asked("NUL") };

async function run(id) {
  await Excel.run(async (context) => {
    const env = { build: Office.context.diagnostics.version, platform: Office.context.diagnostics.platform };
    try {
      if (!CASES[id]) throw { code: "UnknownCase", message: `no case ${id}` };
      // A new, empty workbook only: the sheets this adds and the rows it writes are the evidence.
      const sheets = context.workbook.worksheets;
      sheets.load("items/name");
      await context.sync();
      const used = sheets.items.map((sheet) => sheet.getUsedRangeOrNullObject(true));
      used.forEach((u) => u.load("address"));
      await context.sync();
      const full = used.filter((u) => !u.isNullObject).map((u) => u.address);
      if (full.length || sheets.items.length !== 1) {
        const found = `${sheets.items.length} sheet(s); ${full.join(", ")}`;
        throw { code: "WorkbookNotEmpty", message: `run in a new, empty workbook: ${found}` };
      }
      log({ case: id, ...env, ...(await CASES[id](context, sheets.items[0])) });
    } catch (e) {
      log({ case: id, ...env, error: e.code || String(e), message: e.message });
    }
  });
}

// ---- In Script Lab ---------------------------------------------------------------------------------------------
// This file is the Script tab, and the HTML tab holds one line: <div id="cases"></div><pre id="out"></pre>
// Press Run, then click one case's button: the click turns both off, since a workbook takes one case. Its JSON
// lines are logged, and shown under the buttons to copy from. The other case needs a new workbook.
if (typeof document !== "undefined" && document.getElementById("cases")) {
  const print = console.log;
  console.log = (line) => {
    print(line);
    document.getElementById("out").textContent += `${line}\n`;
  };
  const buttons = [];
  for (const id of Object.keys(CASES)) {
    const button = document.createElement("button");
    button.textContent = id;
    button.onclick = () => {
      buttons.forEach((other) => (other.disabled = true));
      return run(id);
    };
    buttons.push(button);
    document.getElementById("cases").appendChild(button);
  }
}
