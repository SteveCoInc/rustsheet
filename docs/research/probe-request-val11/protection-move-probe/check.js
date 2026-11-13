// Checks cases.js without Excel: node check.js
//
// It runs the script against a fake of the parts of Office.js it uses, with a model of a sheet's rows, its
// allow-edit ranges and one SUM. Between SETUP and READ it makes README.md's five moves on the model, under two
// rules for what a move does (the flags travel with the rows and the ranges follow them; the flags and the
// ranges stay where they were), and holds READ to what the model then is. It says nothing about which rule is
// Excel's: that is what the request asks. Also checked: what SETUP builds; a host with no allow-edit ranges
// still answers for the flags and the SUM; READ before SETUP and SETUP twice are refused and write nothing; any
// one sync failing ends the run normally; the two buttons.
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vm = require("vm");
const { fakePage, checkButtons } = require("../script_lab_page");

const source = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");
const readme = fs.readFileSync(path.join(__dirname, "README.md"), "utf8");

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

const plainRow = () => ({ label: "", b: "", locked: true, hidden: false });
// "A10:C14" -> { column: "A", first: 10, last: 14 }
function span(address) {
  const m = /^([A-E])(\d+)(?::([A-E])(\d+))?$/.exec(address);
  assert.ok(m, `the fake reads no range ${address}`);
  return { column: m[1], first: Number(m[2]), last: Number(m[4] || m[2]) };
}

// The host. A sheet is { name, rows, grants: [{ title, first, last }], sum: { first, last } | null }, with rows
// from index 1. Queued steps take effect at sync; `failAt` makes that sync (from 1) reject.
function makeHost({ used = false, failAt = 0, noGrants = false, gate = undefined } = {}) {
  const sheet0 = { name: "Sheet1", rows: [null], grants: [], sum: null };
  if (used) sheet0.rows[1] = { ...plainRow(), label: "x" };
  const host = { book: [sheet0], syncs: 0, writes: 0, lines: [] };
  let queue = [];
  const rowOf = (sheet, n) => (sheet.rows[n] ||= plainRow());
  const write = (step) =>
    queue.push(() => {
      host.writes += 1;
      step();
    });

  function range(sheet, address) {
    const { column, first, last } = span(address);
    const loaded = {};
    const protection = strict("format protection", {
      load(what) {
        assert.strictEqual(what, "locked,formulaHidden");
        assert.strictEqual(first, last, "the flags are read a cell at a time");
        queue.push(() => {
          const { locked, hidden } = rowOf(sheet, first);
          Object.assign(loaded, { locked, hidden });
        });
      },
      get locked() {
        assert.ok("locked" in loaded, "locked is read after a load and a sync");
        return loaded.locked;
      },
      set locked(v) {
        write(() => {
          for (let n = first; n <= last; n++) rowOf(sheet, n).locked = v;
        });
      },
      get formulaHidden() {
        assert.ok("hidden" in loaded, "formulaHidden is read after a load and a sync");
        return loaded.hidden;
      },
      set formulaHidden(v) {
        write(() => {
          for (let n = first; n <= last; n++) rowOf(sheet, n).hidden = v;
        });
      },
    });
    const sumFormula = () => (sheet.sum ? `=SUM(B${sheet.sum.first}:B${sheet.sum.last})` : "");
    const sumValue = () => {
      let total = 0;
      for (let n = sheet.sum.first; n <= sheet.sum.last; n++) total += Number(rowOf(sheet, n).b) || 0;
      return total;
    };
    return strict("range", {
      format: strict("format", { protection }),
      load(what) {
        queue.push(() => {
          for (const k of what.split(",")) {
            assert.ok(k === "values" || k === "formulas", `the fake range does not load ${k}`);
            if (column === "E") loaded[k] = [[k === "values" ? sumValue() : sumFormula()]];
            else {
              assert.strictEqual(column, "A", "labels are read from column A");
              loaded[k] = [];
              for (let n = first; n <= last; n++) loaded[k].push([rowOf(sheet, n).label]);
            }
          }
        });
      },
      get values() {
        assert.ok("values" in loaded, "values are read after a load and a sync");
        return loaded.values;
      },
      set values(v) {
        assert.strictEqual(v.length, last - first + 1, `${address}: a row of the array for each row of the range`);
        assert.ok(v.every((row) => row.length === 1), `${address}: one column`);
        write(() => {
          v.forEach(([value], i) => {
            rowOf(sheet, first + i)[column === "A" ? "label" : "b"] = value;
          });
        });
      },
      get formulas() {
        assert.ok("formulas" in loaded, "formulas are read after a load and a sync");
        return loaded.formulas;
      },
      set formulas(v) {
        const m = /^=SUM\(B(\d+):B(\d+)\)$/.exec(v[0][0]);
        assert.ok(column === "E" && m, `the fake takes one SUM in E1, not ${v[0][0]} in ${address}`);
        write(() => {
          sheet.sum = { first: Number(m[1]), last: Number(m[2]) };
        });
      },
    });
  }

  function worksheet(sheet) {
    const grants = strict("allow-edit ranges", {
      items: undefined,
      add(title, address) {
        const { first, last } = span(address);
        write(() => {
          if (noGrants) throw Object.assign(new Error("ApiNotFound"), { code: "ApiNotFound" });
          sheet.grants.push({ title, first, last });
        });
      },
      load(what) {
        assert.strictEqual(what, "items/title,items/address");
        queue.push(() => {
          if (noGrants) throw Object.assign(new Error("ApiNotFound"), { code: "ApiNotFound" });
          grants.items = sheet.grants.map((g) => ({ title: g.title, address: `${sheet.name}!A${g.first}:C${g.last}` }));
        });
      },
    });
    return strict("worksheet", {
      name: sheet.name,
      protection: strict("worksheet protection", { allowEditRanges: grants }),
      getRange: (address) => range(sheet, address),
      getUsedRangeOrNullObject(valuesOnly) {
        assert.strictEqual(valuesOnly, true);
        const state = {};
        return strict("used range", {
          load(what) {
            assert.strictEqual(what, "address");
            queue.push(() => {
              state.isNullObject = !sheet.rows.some((row) => row && (row.label !== "" || row.b !== ""));
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
        worksheets.items = host.book.map(worksheet);
      });
    },
    add(name) {
      const sheet = { name, rows: [null], grants: [], sum: null };
      write(() => {
        assert.ok(!host.book.some((s) => s.name === name), `a second sheet ${name}`);
        host.book.push(sheet);
      });
      return worksheet(sheet);
    },
    getItem(name) {
      const sheet = host.book.find((s) => s.name === name);
      assert.ok(sheet, `the fake has no sheet ${name}`);
      return worksheet(sheet);
    },
  });
  const context = strict("context", {
    workbook: strict("workbook", { worksheets }),
    async sync() {
      host.syncs += 1;
      const steps = queue;
      queue = [];
      if (host.syncs === failAt) throw Object.assign(new Error("Injected"), { code: "Injected" });
      steps.forEach((step) => step());
    },
  });
  host.sandbox = {
    console: { log: (line) => host.lines.push(line) },
    Office: { context: { diagnostics: { version: "16.0.FAKE", platform: "OfficeOnline" } } },
    Excel: {
      run: async (f) => {
        await gate; // a test holds a run here, to look at the page while the case runs
        return f(context);
      },
    },
  };
  vm.createContext(host.sandbox);
  vm.runInContext(`${source}\n;globalThis.__run = run; globalThis.__moves = MOVES;`, host.sandbox);
  host.run = async (id) => {
    const before = host.lines.length;
    await host.sandbox.__run(id);
    const lines = host.lines.slice(before).map((line) => JSON.parse(line));
    assert.strictEqual(lines.length, 1, `${id}: one line`);
    return lines[0];
  };
  return host;
}

// A person's cut-and-insert on the model: rows `first` to `last` are cut and inserted above row `above`.
// `travel` says whether the flags go with the rows, and whether a range follows the rows it named: a range
// wholly inside the cut rows goes with them, one that loses rows shrinks, one that gains rows inside it grows.
function move(sheet, first, last, above, travel) {
  const count = last - first + 1;
  const rows = Array.from({ length: 40 }, (_, i) => sheet.rows[i + 1] || plainRow());
  const flags = rows.map((row) => ({ locked: row.locked, hidden: row.hidden }));
  const cut = rows.splice(first - 1, count);
  const at = above > last ? above - 1 - count : above - 1; // where the cut rows go, from 0, once they are out
  rows.splice(at, 0, ...cut);
  if (!travel) rows.forEach((row, i) => Object.assign(row, flags[i]));
  sheet.rows = [null, ...rows];
  if (!travel) return;
  const where = (n) => {
    if (n >= first && n <= last) return at + 1 + (n - first); // a cut row
    const out = n > last ? n - count : n; // its row once the cut rows are out
    return out > at ? out + count : out;
  };
  for (const r of [...sheet.grants, ...(sheet.sum ? [sheet.sum] : [])]) {
    const inside = (n) => n >= first && n <= last;
    if (inside(r.first) && inside(r.last)) Object.assign(r, { first: where(r.first), last: where(r.last) });
    else {
      // The rows of the range that stay, in their new places: the range is from the first to the last of them.
      const kept = [];
      for (let n = r.first; n <= r.last; n++) if (!inside(n)) kept.push(where(n));
      Object.assign(r, { first: Math.min(...kept), last: Math.max(...kept) });
    }
  }
}

// What READ must say of a sheet of the model.
function expected(sheet) {
  const rows = [];
  for (let n = 1; n <= 36; n++) {
    const row = sheet.rows[n] || plainRow();
    const plain = row.label === "" && row.locked === true && row.hidden === false;
    if (!plain) rows.push([n, row.label, row.locked, row.hidden]);
  }
  let total = 0;
  for (let n = sheet.sum.first; n <= sheet.sum.last; n++) total += Number((sheet.rows[n] || plainRow()).b) || 0;
  return {
    rows,
    grants: sheet.grants.map((g) => [g.title, `${sheet.name}!A${g.first}:C${g.last}`]),
    sum: [`=SUM(B${sheet.sum.first}:B${sheet.sum.last})`, total],
  };
}

(async () => {
  // README.md's moves: a sheet, the rows cut, the row they are inserted above.
  const moves = [...readme.matchAll(/^\| `([A-Z]+)` \| rows? (\d+)(?: to (\d+))? \| row (\d+) \|/gm)].map((m) => ({
    sheet: m[1],
    first: Number(m[2]),
    last: Number(m[3] || m[2]),
    above: Number(m[4]),
  }));

  // The moves written out: an inner row, the whole block, its first row, its last row, a row from outside.
  const asked = "INNER 12 12 25, WHOLE 10 14 25, FIRST 10 10 25, LAST 14 14 25, INTO 25 25 12";
  assert.strictEqual(moves.map((m) => `${m.sheet} ${m.first} ${m.last} ${m.above}`).join(", "), asked);

  // SETUP on a new workbook.
  const fresh = makeHost();
  const names = [...fresh.sandbox.__moves];
  assert.deepStrictEqual(moves.map((m) => m.sheet), names, "README.md lists the script's sheets, in order");
  const setup = await fresh.run("SETUP");
  const added = Object.fromEntries(names.map((name) => [name, { ok: true }]));
  assert.deepStrictEqual(setup.data, { sheets: names, block: "A10:C14", grants: added });
  assert.deepStrictEqual(fresh.book.map((s) => s.name), ["Sheet1", ...names]);
  for (const sheet of fresh.book.slice(1)) {
    for (let n = 1; n <= 30; n++) {
      const block = n >= 10 && n <= 14;
      const want = { label: `r${n}`, b: 1, locked: !block, hidden: block };
      assert.deepStrictEqual(sheet.rows[n], want, `${sheet.name} row ${n}`);
    }
    assert.strictEqual(sheet.rows.length, 31, `${sheet.name}: 30 rows`);
    assert.deepStrictEqual(sheet.grants, [{ title: "Grant", first: 10, last: 14 }], `${sheet.name}: the grant`);
    assert.deepStrictEqual(sheet.sum, { first: 10, last: 14 }, `${sheet.name}: the SUM`);
  }

  // The moves, under each rule, and READ.
  for (const travel of [true, false]) {
    const host = makeHost();
    await host.run("SETUP");
    for (const m of moves) move(host.book.find((s) => s.name === m.sheet), m.first, m.last, m.above, travel);
    // A row with no label whose flags are not a plain cell's is reported too.
    host.book[1].rows[33] = { ...plainRow(), locked: false };
    const writes = host.writes;
    const read = await host.run("READ");
    const rule = travel ? "the flags and ranges travel" : "the flags and ranges stay";
    assert.strictEqual(host.writes, writes, `${rule}: READ writes nothing`);
    assert.deepStrictEqual(Object.keys(read.data), names, `${rule}: a sheet a move`);
    for (const sheet of host.book.slice(1)) {
      assert.deepStrictEqual(read.data[sheet.name], expected(sheet), `${rule}: ${sheet.name}`);
    }
    assert.deepStrictEqual(read.data.INNER.rows.at(-1), [33, "", false, false], `${rule}: the row with no label`);
    if (travel) {
      // The model's own rule, spelled out once, so that a slip in `move` does not pass as READ's truth.
      const at = (name) => read.data[name];
      assert.deepStrictEqual(at("WHOLE").grants, [["Grant", "WHOLE!A20:C24"]], "the whole block: the range goes along");
      assert.deepStrictEqual(at("INNER").grants, [["Grant", "INNER!A10:C13"]], "an inner row out: the range shrinks");
      assert.deepStrictEqual(at("INTO").grants, [["Grant", "INTO!A10:C15"]], "a row in: the range grows");
      assert.deepStrictEqual(at("INNER").rows.find(([, label]) => label === "r12"), [24, "r12", false, true]);
      assert.deepStrictEqual(at("INTO").rows.find(([, label]) => label === "r25"), [12, "r25", true, false]);
      assert.deepStrictEqual(at("WHOLE").sum, ["=SUM(B20:B24)", 5]);
    } else {
      assert.deepStrictEqual(read.data.INNER.rows.find(([, label]) => label === "r12"), [24, "r12", true, false]);
      assert.deepStrictEqual(read.data.WHOLE.grants, [["Grant", "WHOLE!A10:C14"]]);
    }
  }

  // A host with no allow-edit ranges: SETUP and READ say so for each sheet and answer for the rest.
  const bare = makeHost({ noGrants: true });
  const refusal = { error: "ApiNotFound", message: "ApiNotFound" };
  const none = Object.fromEntries(names.map((name) => [name, refusal]));
  assert.deepStrictEqual((await bare.run("SETUP")).data.grants, none, "SETUP without allow-edit ranges");
  for (const m of moves) move(bare.book.find((s) => s.name === m.sheet), m.first, m.last, m.above, true);
  const bareRead = await bare.run("READ");
  for (const sheet of bare.book.slice(1)) {
    const want = { ...expected(sheet), grants: refusal };
    assert.deepStrictEqual(bareRead.data[sheet.name], want, `READ without allow-edit ranges: ${sheet.name}`);
  }

  // READ before SETUP, SETUP in a used workbook and SETUP twice: refused, and nothing is written.
  const early = makeHost();
  assert.strictEqual((await early.run("READ")).error, "NotSetUp");
  assert.strictEqual(early.writes, 0, "READ before SETUP writes nothing");
  const usedBook = makeHost({ used: true });
  assert.strictEqual((await usedBook.run("SETUP")).error, "WorkbookNotEmpty");
  assert.strictEqual(usedBook.writes, 0, "SETUP in a used workbook writes nothing");
  const twice = makeHost();
  await twice.run("SETUP");
  const once = twice.writes;
  assert.strictEqual((await twice.run("SETUP")).error, "WorkbookNotEmpty");
  assert.strictEqual(twice.writes, once, "a second SETUP writes nothing");
  for (const sheet of twice.book) sheet.rows = [null]; // the five sheets, emptied: still not a new workbook
  assert.strictEqual((await twice.run("SETUP")).error, "WorkbookNotEmpty");
  assert.strictEqual(twice.writes, once, "SETUP over its own emptied sheets writes nothing");
  assert.strictEqual((await makeHost().run("OTHER")).error, "UnknownCase");

  // Any one sync failing: the run ends normally with its one line, which names the failure: as the line's error,
  // or, where the sync was an allow-edit step's own, as that step's.
  let failures = 0;
  for (const id of ["SETUP", "READ"]) {
    const clean = makeHost();
    if (id === "READ") await clean.run("SETUP");
    const start = clean.syncs;
    await clean.run(id);
    for (let n = start + 1; n <= clean.syncs; n++) {
      const host = makeHost({ failAt: n });
      if (id === "READ") await host.run("SETUP");
      const line = await host.run(id);
      assert.strictEqual(line.case, id, `${id}, sync ${n} fails`);
      const steps = JSON.stringify(line.data || {}).split('{"error":"Injected"').length - 1;
      assert.strictEqual((line.error === "Injected" ? 1 : 0) + steps, 1, `${id}, sync ${n} fails: said once`);
      failures += 1;
    }
  }

  // In Script Lab: a click runs its case and turns off its own button and the one before it.
  const open = () => {
    const page = fakePage();
    const host = makeHost();
    const paged = { ...host.sandbox, document: page.document };
    vm.createContext(paged);
    vm.runInContext(source, paged);
    return { page, logged: () => host.lines };
  };
  const ids = ["SETUP", "READ"];
  await checkButtons(open, ids, (line) => line.case, (i) => ids.map((_, j) => j <= i));
  // While a case runs both buttons are off, so READ cannot start in the middle of SETUP.
  let release;
  const held = makeHost({ gate: new Promise((resolve) => (release = resolve)) });
  const page = fakePage();
  const paged = { ...held.sandbox, document: page.document };
  vm.createContext(paged);
  vm.runInContext(source, paged);
  const off = () => page.buttons.map((button) => button.disabled);
  const setting = page.buttons[0].onclick();
  assert.deepStrictEqual(off(), [true, true], "both buttons are off while SETUP runs");
  release();
  await setting;
  assert.deepStrictEqual(off(), [true, false], "SETUP stays off, and READ comes back");
  await page.buttons[1].onclick();
  assert.deepStrictEqual(off(), [true, true], "after READ both are off");
  assert.deepStrictEqual(held.lines.map((line) => JSON.parse(line).case), ["SETUP", "READ"]);

  const rest = `${failures} syncs failed in turn; refusals write nothing; 2 buttons`;
  console.log(`ok: ${moves.length} moves under 2 rules; ${rest}`);
})().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
