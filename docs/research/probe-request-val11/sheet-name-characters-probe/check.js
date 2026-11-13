// Checks cases.js without Excel: node check.js
//
// It runs the script against a fake of the parts of Office.js it uses, and says nothing about what Excel does
// with a name: that is what the request asks. Checked here:
// - the script's candidates, with their names' code units, are README.md's table, and no two share their letters;
// - on a host that takes every name: one line a candidate, in README.md's order, each with the name read back
//   both ways, the reference's spelling, the formula entered in its row of column B and its value, and a last
//   line with the sheets;
// - on a host that refuses names (at adding, at renaming, at both) or changes them (trims, drops a control
//   character, puts a space in its place, stops at U+0000): the line says so, the candidate's sheet is the one
//   that took the name, no candidate is refused for another's name, and the run goes on;
// - with any one sync failing: the run ends normally, with a last line or an error line, and one line at most
//   for each candidate;
// - where no sheet can be added to rename, the line says that and reports no rename;
// - a workbook that holds a value, or more sheets than one, is refused and left untouched;
// - in Script Lab a click on a case's button runs it and turns both buttons off.
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vm = require("vm");
const { fakePage, checkButtons } = require("../script_lab_page");

const source = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");
const readme = fs.readFileSync(path.join(__dirname, "README.md"), "utf8");

const text = (units) => String.fromCharCode(...units);
// How the fake spells a reference to a sheet: quoted unless the name is letters, digits and underscores.
const spelled = (name) => (/^[A-Za-z_][A-Za-z0-9_]*$/.test(name) ? name : `'${name.replace(/'/g, "''")}'`);

// An object that fails on any member the fake does not have, so that a call the check never made is not taken.
function strict(what, members) {
  return new Proxy(members, {
    get(target, key) {
      if (key in target || typeof key === "symbol" || key === "then") return target[key];
      throw new Error(`the fake ${what} has no ${String(key)}`);
    },
    set(target, key, value) {
      if (!(key in target)) throw new Error(`the fake ${what} has no ${String(key)} to set`);
      target[key] = value;
      return true;
    },
  });
}

// The host. Queued steps take effect at sync, in order; a step the host refuses makes that sync reject with its
// code and drops the steps queued after it, as Office.js does. `refuse(name, how)` gives a code or null, with
// `how` "add" or "rename"; `store(name)` is the name the host keeps; `failAt` makes that sync (from 1) reject.
function makeHost(options = {}) {
  const { refuse = () => null, store = (name) => name, used = false, sheets = 1, failAt = 0 } = options;
  const { noNewSheet = false } = options; // a host on which a sheet with no name asked for cannot be added
  const book = Array.from({ length: sheets }, (_, i) => ({ name: `Sheet${i + 1}`, cells: {} }));
  if (used) book[0].cells.A1 = 1;
  const host = { book, syncs: 0, writes: 0, lines: [] };
  let queue = [];
  let made = sheets;
  const fail = (code) => Object.assign(new Error(code), { code });
  const taken = (name, but) => book.some((s) => s !== but && s.name.toLowerCase() === name.toLowerCase());

  function range(sheet, address) {
    const loaded = {};
    const cell = { address: undefined, values: undefined, formulas: undefined };
    return strict("range", {
      load(what) {
        queue.push(() => {
          for (const k of what.split(",")) {
            if (k === "address") loaded.address = `${spelled(sheet.name)}!${address}`;
            else if (k === "values") loaded.values = [[sheet.cells[address]]];
            else if (k === "formulas") loaded.formulas = [[sheet.formulas?.[address] ?? sheet.cells[address]]];
            else throw new Error(`the fake range does not load ${k}`);
          }
        });
      },
      get address() {
        assert.ok("address" in loaded, "address is read after a load and a sync");
        return loaded.address;
      },
      get values() {
        assert.ok("values" in loaded, "values are read after a load and a sync");
        return loaded.values;
      },
      set values(v) {
        assert.ok(Array.isArray(v) && v.length === 1 && v[0].length === 1, "one cell is written as [[v]]");
        queue.push(() => {
          host.writes += 1;
          sheet.cells[address] = v[0][0];
        });
        cell.values = v;
      },
      get formulas() {
        assert.ok("formulas" in loaded, "formulas are read after a load and a sync");
        return loaded.formulas;
      },
      set formulas(v) {
        assert.ok(Array.isArray(v) && v.length === 1 && v[0].length === 1, "one cell is written as [[f]]");
        queue.push(() => {
          const target = book.find((s) => v[0][0] === `=${spelled(s.name)}!A1`);
          if (!target) throw fail("InvalidArgument");
          host.writes += 1;
          sheet.formulas = { ...sheet.formulas, [address]: v[0][0] };
          sheet.cells[address] = target.cells.A1;
        });
        cell.formulas = v;
      },
    });
  }

  function worksheet(sheet) {
    let loaded;
    return strict("worksheet", {
      load(what) {
        assert.strictEqual(what, "name");
        queue.push(() => {
          loaded = sheet.name;
        });
      },
      get name() {
        assert.ok(loaded !== undefined, "a name is read after a load and a sync");
        return loaded;
      },
      set name(name) {
        queue.push(() => {
          const code = refuse(name, "rename") || (taken(store(name), sheet) ? "ItemAlreadyExists" : null);
          if (code) throw fail(code);
          host.writes += 1;
          sheet.name = store(name);
        });
      },
      delete() {
        queue.push(() => {
          host.writes += 1;
          book.splice(book.indexOf(sheet), 1);
        });
      },
      getRange: (address) => range(sheet, address),
      getUsedRangeOrNullObject(valuesOnly) {
        assert.strictEqual(valuesOnly, true);
        const state = {};
        return strict("used range", {
          load(what) {
            assert.strictEqual(what, "address");
            queue.push(() => {
              state.isNullObject = Object.keys(sheet.cells).length === 0;
              state.address = `${sheet.name}!A1`;
            });
          },
          get isNullObject() {
            return state.isNullObject;
          },
          get address() {
            return state.address;
          },
        });
      },
    });
  }

  const worksheets = strict("worksheets", {
    items: undefined,
    load(what) {
      assert.strictEqual(what, "items/name");
      queue.push(() => {
        worksheets.items = book.map((sheet) => {
          const one = worksheet(sheet);
          one.load("name");
          return one;
        });
      });
    },
    add(name) {
      const sheet = { name: undefined, cells: {} };
      queue.push(() => {
        const wanted = name === undefined ? `Sheet${(made += 1)}` : name;
        const exists = () => (taken(store(name)) ? "ItemAlreadyExists" : null);
        const unnamed = noNewSheet ? "GeneralException" : null;
        const code = name === undefined ? unnamed : refuse(name, "add") || exists();
        if (code) throw fail(code);
        host.writes += 1;
        sheet.name = name === undefined ? wanted : store(name);
        book.push(sheet);
      });
      return worksheet(sheet);
    },
  });
  const context = strict("context", {
    workbook: strict("workbook", { worksheets }),
    async sync() {
      host.syncs += 1;
      const steps = queue;
      queue = [];
      if (host.syncs === failAt) throw fail("Injected");
      // A load queued inside a step (the sheets' names) runs in the same sync.
      for (let i = 0; i < steps.length; i++) {
        steps[i]();
        steps.push(...queue.splice(0));
      }
    },
  });
  host.sandbox = {
    console: { log: (line) => host.lines.push(line) },
    Office: { context: { diagnostics: { version: "16.0.FAKE", platform: "OfficeOnline" } } },
    Excel: { run: async (f) => f(context) },
  };
  return host;
}

async function run(options, id = "NAMES") {
  const host = makeHost(options);
  vm.createContext(host.sandbox);
  vm.runInContext(`${source}\n;globalThis.__run = run; globalThis.__candidates = CANDIDATES;`, host.sandbox);
  await host.sandbox.__run(id);
  const lines = host.lines.map((line) => JSON.parse(line));
  const all = Object.entries(host.sandbox.__candidates).map(([which, list]) => [which, [...list].map((c) => [...c])]);
  return { host, lines, all: Object.fromEntries(all) };
}

(async () => {
  // README.md's table: every candidate of both cases, in the script's order, with its name's code units.
  const { all } = await run({}, "OTHER");
  assert.deepStrictEqual(Object.keys(all), ["NAMES", "NUL"], "the script's cases");
  const listed = [...readme.matchAll(/^\| `([a-z0-9-]+)` \| .* \| `([0-9 ]+)` \|/gm)].map((m) => [m[1], m[2]]);
  const unitsOf = (name) => Array.from({ length: name.length }, (_, i) => name.charCodeAt(i)).join(" ");
  assert.deepStrictEqual(
    [...all.NAMES, ...all.NUL].map(([id, name]) => [id, unitsOf(name)]),
    listed,
    "the script's candidates and their code units are README.md's, in order",
  );
  // U+0000 is asked in a workbook of its own, and nowhere else.
  assert.deepStrictEqual(all.NUL.map(([id]) => id), ["u0000"]);
  assert.ok(all.NUL[0][1].includes("\u0000") && !all.NAMES.some(([, name]) => name.includes("\u0000")));
  // No two names hold the same letters: with every character that is not a letter taken out, all differ.
  const letters = [...all.NAMES, ...all.NUL].map(([, name]) => name.replace(/[^A-Za-z\u{10000}-\u{10FFFF}]/gu, ""));
  const lettered = letters.filter((l) => l !== "");
  assert.strictEqual(new Set(lettered).size, lettered.length, "no two candidates share their letters");

  // A host that takes every name, for each case.
  let syncs = 0;
  for (const [which, candidates] of Object.entries(all)) {
    const { host, lines } = await run({}, which);
    if (which === "NAMES") syncs = host.syncs;
    assert.strictEqual(lines.length, candidates.length + 1, `${which}: one line a candidate, and the last`);
    candidates.forEach(([id, name], i) => {
      const line = lines[i];
      const where = `candidate ${id}`;
      assert.deepStrictEqual([line.case, line.candidate], [which, id], where);
      assert.strictEqual(text(line.asked), name, `${where}: the name asked for, in code units`);
      assert.deepStrictEqual(line.added, { ok: line.asked }, `${where}: added`);
      assert.deepStrictEqual(line.addedDeleted, { ok: true }, `${where}: the added sheet is deleted`);
      assert.deepStrictEqual(line.renamed, { ok: line.asked }, `${where}: renamed`);
      assert.strictEqual(line.sheet, "renamed", where);
      assert.strictEqual(text(line.address.ok), `${spelled(name)}!A1`, `${where}: the reference as the host spells it`);
      assert.strictEqual(text(line.reader.ok.formula), `=${spelled(name)}!A1`, `${where}: the formula read back`);
      assert.strictEqual(line.reader.ok.value, i + 1, `${where}: the formula reads the sheet's A1`);
      // The formula is in column B of the first sheet, on the candidate's row, as README.md says.
      assert.strictEqual(host.book[0].formulas[`B${i + 1}`], `=${spelled(name)}!A1`, `${where}: in B${i + 1}`);
    });
    const last = lines[lines.length - 1];
    assert.deepStrictEqual([last.case, last.build, last.platform], [which, "16.0.FAKE", "OfficeOnline"]);
    assert.deepStrictEqual(last.sheets.map(text), ["Sheet1", ...candidates.map(([, name]) => name)]);
    assert.deepStrictEqual(host.book.slice(1).map((s) => s.cells.C1), candidates.map(([id]) => id), "C1 is the id");
  }

  // Hosts that refuse or change names. A control character is refused at adding, at renaming, or at both; a
  // blank name is refused; names are trimmed; and three hosts that change a name where the others refuse it,
  // which would make two candidates one name if any two shared their letters.
  const control = (name) => /[\u0000-\u001f\u007f]/.test(name);
  const controls = /[\u0000-\u001f\u007f]/g;
  const policies = {
    "refused at adding": { refuse: (name, how) => (how === "add" && control(name) ? "InvalidArgument" : null) },
    "refused at renaming": { refuse: (name, how) => (how === "rename" && control(name) ? "InvalidArgument" : null) },
    "refused both ways": { refuse: (name) => (control(name) || !name.trim() ? "InvalidArgument" : null) },
    trimmed: { store: (name) => name.trim() || name },
    "control characters dropped": { store: (name) => name.replace(controls, "") || name },
    "control characters made spaces": { store: (name) => name.replace(controls, " ") },
    "trimmed, and cut at U+0000": { store: (name) => name.split("\u0000")[0].trim() || name },
  };
  for (const [what, policy] of Object.entries(policies)) {
    for (const [which, candidates] of Object.entries(all)) {
      const result = await run(policy, which);
      assert.strictEqual(result.lines.length, candidates.length + 1, `${what}: every candidate has its line`);
      candidates.forEach(([id, name], i) => {
        const line = result.lines[i];
        const where = `${what}, candidate ${id}`;
        const refuse = policy.refuse || (() => null);
        const store = policy.store || ((n) => n);
        const stored = Array.from({ length: store(name).length }, (_, k) => store(name).charCodeAt(k));
        for (const [how, got] of [["add", line.added], ["rename", line.renamed]]) {
          if (refuse(name, how)) assert.strictEqual(got.error, "InvalidArgument", `${where}: refused at ${how}`);
          else assert.deepStrictEqual(got, { ok: stored }, `${where}: ${how}`);
        }
        const kept = "ok" in line.renamed ? "renamed" : "ok" in line.added ? "added again" : "not named";
        assert.strictEqual(line.sheet, kept, `${where}: the candidate's sheet`);
        const sheet = result.host.book.find((s) => s.cells.C1 === id);
        assert.ok(sheet, `${where}: a sheet holds the id`);
        if (kept === "not named") assert.match(sheet.name, /^Sheet\d+$/, `${where}: it keeps the host's name`);
        else assert.strictEqual(sheet.name, store(name), `${where}: its name`);
        assert.strictEqual(text(line.address.ok), `${spelled(sheet.name)}!A1`, `${where}: the reference`);
        assert.strictEqual(line.reader.ok.value, i + 1, `${where}: the formula reads the sheet's A1`);
      });
      assert.strictEqual(result.host.book.length, candidates.length + 1, `${what}: one sheet a candidate`);
    }
  }

  // A host on which no sheet can be added for the rename: the line says so, and says nothing of a rename.
  const stuck = await run({ noNewSheet: true });
  for (const line of stuck.lines.slice(0, -1)) {
    assert.deepStrictEqual(line.added, { ok: line.asked }, `${line.candidate}: added all the same`);
    assert.strictEqual(line.fresh.error, "GeneralException", `${line.candidate}: no sheet to rename`);
    assert.ok(!("renamed" in line) && !("sheet" in line), `${line.candidate}: no rename is reported`);
  }
  assert.strictEqual(stuck.lines.length, all.NAMES.length + 1);

  // Any one sync failing: the run ends normally, with a last line or an error line.
  const candidates = all.NAMES;
  for (let n = 1; n <= syncs; n++) {
    const result = await run({ failAt: n });
    const where = `sync ${n} of ${syncs} fails`;
    const end = result.lines[result.lines.length - 1];
    assert.strictEqual(end.case, "NAMES", where);
    assert.ok("sheets" in end || "error" in end, `${where}: a last line or an error line`);
    const ids = result.lines.slice(0, -1).map((line) => line.candidate);
    assert.deepStrictEqual(ids, candidates.map(([id]) => id).slice(0, ids.length), `${where}: candidates in order`);
    if (n > 2) assert.strictEqual(ids.length, candidates.length, `${where}: every candidate still has its line`);
  }

  // A workbook that is not new and empty is refused, untouched, by either case; so is an unknown case.
  for (const which of Object.keys(all)) {
    for (const [what, options] of [["a value", { used: true }], ["two sheets", { sheets: 2 }]]) {
      const result = await run(options, which);
      assert.strictEqual(result.lines.length, 1, `${which}, ${what}: one line`);
      assert.strictEqual(result.lines[0].error, "WorkbookNotEmpty", `${which}, ${what}: refused`);
      assert.strictEqual(result.host.writes, 0, `${which}, ${what}: nothing is written`);
    }
  }
  const unknown = await run({}, "OTHER");
  assert.deepStrictEqual([unknown.lines[0].error, unknown.host.syncs], ["UnknownCase", 0]);

  // In Script Lab: a button a case; a click runs its case, shows its lines, and turns both buttons off.
  const open = () => {
    const page = fakePage();
    const paged = makeHost();
    paged.sandbox.document = page.document;
    vm.createContext(paged.sandbox);
    vm.runInContext(source, paged.sandbox);
    return { page, logged: () => paged.lines };
  };
  await checkButtons(open, Object.keys(all), (line) => line.case);

  const hosts = 1 + Object.keys(policies).length;
  const rest = `${syncs} syncs failed in turn; a used workbook refused; 2 buttons`;
  console.log(`ok: ${listed.length} candidates in 2 cases on ${hosts} hosts; ${rest}`);
})().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
