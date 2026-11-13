// Checks the sequencing of cases.js without Excel: node check.js
//
// It runs every M case against a recording fake of Office.js and asserts what the cases depend on. It says nothing
// about Excel's values: those are what the request asks for. Checked here:
// - every M case logs exactly one line, with no error;
// - every case's line says whether iterative calculation was on when it started;
// - M20 (a loop closed in Manual mode) turns iterative calculation off before it closes the loop, keeps it off
//   through F9, logs both settings and puts the first one back. It is run from a host with iteration on and from
//   one with it off (review of CR-321738056, round 1: on the desktop the setting is the application's);
// - M20 restores iteration after injected sync failures except when its restoration write is dropped; if
//   run()'s cleanup sync succeeds, the mode is Automatic afterwards;
// - PREC and DEP build their workbook with no refused group and make exactly the queries listed below: every
//   cell and range the README names for them, with both calls or the direct one alone (review of CR-321841262,
//   round 1: a list of some of them let 41 queries be removed unseen);
// - a workbook that holds a value is refused: one line with WorkbookNotFresh, and nothing is set or calculated;
// - in Script Lab the script puts one button a case on the page, the first and the last each run their own
//   case, and a click turns every button off (../script_lab_page.js).
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vm = require("vm");
const { fakePage, checkButtons } = require("../script_lab_page");

const src = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");

// A host whose application remembers the calculation mode and the iteration setting; every other property is a
// permissive fake. A write of one of the two settings takes effect at the next sync, as in Office.js. Each set and
// each call is recorded with the two settings as they were at that moment. `fail`, if given, makes the sync with
// that number (from 1) reject once, after applying the queued writes (`applied`) or dropping them. With `used`
// the workbook holds a value already: the second of its two sheets has a used range.
function makeHost(iterationOn, fail, used) {
  const state = { iteration: iterationOn, mode: "Automatic" };
  const host = { syncs: 0 };
  let queued = [];
  const sync = async () => {
    host.syncs += 1;
    const failing = fail && host.syncs === fail.sync;
    if (!failing || fail.applied) for (const [k, v] of queued) state[k] = v;
    queued = [];
    if (failing) throw Object.assign(new Error("injected sync failure"), { code: "Injected" });
  };
  const timeline = [];
  const cache = new Map();
  const special = (p, k) =>
    (p === "context.application.iterativeCalculation" && k === "enabled" && "iteration") ||
    (p === "context.application" && k === "calculationMode" && "mode");
  function fake(p) {
    if (cache.has(p)) return cache.get(p);
    const store = {};
    const proxy = new Proxy(function () {}, {
      get(_target, k) {
        if (k === "then") return undefined;
        if (k === "toJSON") return () => `<${p}>`;
        if (k === Symbol.toPrimitive) return () => p;
        if (special(p, k)) return state[special(p, k)];
        // Two sheets, and the value is on the second: a guard that looked at the active sheet alone would miss it.
        if (p === "context.workbook.worksheets" && k === "items") return [fake(`${p}.items[0]`), fake(`${p}.items[1]`)];
        if (p.endsWith(".getUsedRangeOrNullObject()") && k === "isNullObject") return !(used && p.includes("items[1]"));
        if (p.endsWith(".getUsedRangeOrNullObject()") && k === "address") return "Sheet2!A1";
        if (k in store) return store[k];
        if (k === "sync") return sync;
        return fake(`${p}.${String(k)}`);
      },
      set(_target, k, v) {
        if (special(p, k)) queued.push([special(p, k), v]);
        else store[k] = v;
        timeline.push({ set: `${p}.${String(k)}`, value: v, ...state });
        return true;
      },
      apply(_target, _this, args) {
        timeline.push({ call: p, args, ...state });
        return fake(`${p}()`);
      },
    });
    cache.set(p, proxy);
    return proxy;
  }
  return Object.assign(host, { context: fake("context"), state, timeline });
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
      CalculationType: { recalculate: "Recalculate" },
      RangeCopyType: { formulas: "Formulas" },
      InsertShiftDirection: { down: "Down" },
      ConditionalFormatType: { custom: "Custom" },
    },
  };
  vm.createContext(sandbox);
  const exports = "globalThis.__run = run; globalThis.__ids = Object.keys(M); globalThis.__p = Object.keys(P);";
  vm.runInContext(`${src}\n;${exports}`, sandbox);
  return sandbox;
}

async function runCase(id, iterationOn, fail) {
  const host = makeHost(iterationOn, fail);
  const lines = [];
  const sandbox = load(host, lines);
  let rejected = null;
  await sandbox.__run(id).catch((e) => {
    rejected = e; // only the sync of run()'s own cleanup can make run() reject
  });
  return { host, lines, ids: sandbox.__ids, rejected };
}

(async () => {
  const { ids } = await runCase("M1", false);
  // The cases the README's M table asks through the script (M9 and M10 are done by hand), written out: a case
  // dropped from the script must fail here, not shorten the run.
  const expected = "M1 M2 M3 M4 M6 M8 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 V1".split(" ");
  assert.deepStrictEqual([...ids], expected, "the script holds exactly the README's cases");
  for (const id of ids) {
    for (const iterationOn of [false, true]) {
      const { host, lines } = await runCase(id, iterationOn);
      assert.strictEqual(lines.length, 1, `${id}: one line`);
      assert.strictEqual(lines[0].id, id);
      assert.ok(!("error" in lines[0]), `${id}: ${lines[0].error} ${lines[0].message}`);
      assert.strictEqual(lines[0].iterationAtStart, iterationOn, `${id}: logs the iteration setting it found`);
      assert.strictEqual(host.state.mode, "Automatic", `${id}: leaves Automatic behind`);
      assert.strictEqual(host.state.iteration, iterationOn, `${id}: leaves the iteration setting as it found it`);
      if (id !== "M20") continue;
      const closes = host.timeline.find((e) => e.set && e.set.endsWith(".formulas") && e.value[0][0] === "=A1");
      assert.ok(closes, "M20 closes the loop");
      assert.strictEqual(closes.iteration, false, "M20 must disable iteration before closing the cycle");
      assert.strictEqual(closes.mode, "Manual", "M20 closes the loop in Manual mode");
      const after = host.timeline.slice(host.timeline.indexOf(closes));
      const f9 = after.find((e) => e.call === "context.application.calculate");
      assert.ok(f9, "M20 calculates after the entry");
      assert.strictEqual(f9.iteration, false, "M20 keeps iteration off through F9");
      assert.strictEqual(lines[0].data.iterationAtStart, iterationOn, "M20 logs the setting it found");
      assert.strictEqual(lines[0].data.iterationInCase, false, "M20 logs the setting it ran under");
    }
  }
  // M20 under one failed sync. Its syncs are numbered in a clean run; the last is run()'s cleanup and the one
  // before it M20's own restore. The script does not recover if the restore's own sync drops its write, so that
  // combination is outside this check's restoration guarantee.
  let failures = 0;
  for (const iterationOn of [false, true]) {
    const total = (await runCase("M20", iterationOn)).host.syncs;
    for (let n = 1; n <= total; n++) {
      for (const applied of [false, true]) {
        if (n === total - 1 && !applied) continue;
        const { host, lines, rejected } = await runCase("M20", iterationOn, { sync: n, applied });
        const setting = iterationOn ? "on" : "off";
        const writes = applied ? "applied" : "dropped";
        const where = `M20, iteration ${setting}, sync ${n} of ${total} fails, writes ${writes}`;
        assert.strictEqual(host.state.iteration, iterationOn, `${where}: M20 must restore iteration`);
        if (n < total) {
          assert.ok(!rejected, `${where}: run() handles the failure`);
          assert.strictEqual(host.state.mode, "Automatic", `${where}: Automatic afterwards`);
          assert.strictEqual(lines.length, 1, `${where}: one line`);
          if (n < total - 1) assert.ok("error" in lines[0], `${where}: the line carries the error`);
        }
        failures += 1;
      }
    }
  }
  // The P cases: the builder refuses nothing on the fake, and the script asks exactly the queries listed here,
  // no fewer and no more. The list is written out, not taken from the script: every cell and range of the
  // README's P table and of its "Dependents asked for", and A1 and R10 in PREC, which the script also asks.
  // "direct" marks a query for which only the direct call is made.
  const h = Array.from({ length: 23 }, (_, i) => `H${i + 1}`);
  const cells = (sheet, names) => names.split(" ").map((a) => `${sheet}${a}`);
  const asked = {
    PREC: { both: [...h, ...cells("", "J1 J2 K1 K2 K3 A1 R10 S10 V3 W2 W5 Y1 Y2 X4:X5")], direct: ["H1:H2"] },
    DEP: {
      both: [
        ...cells("S!", "A1 A2 A3 A5 A9 B1 B2 B3 B5 C1 C2 C3 D1 J1 J2 J3 N2 H1"),
        ...cells("S!", "R1 R10 R11 X1 X2 X4:X5"),
      ],
      direct: ["T!A1", "S!A1:A2"],
    },
  };
  for (const [id, { both, direct }] of Object.entries(asked)) {
    const { lines } = await runCase(id, false);
    assert.strictEqual(lines.length, 1, `${id}: one line`);
    assert.ok(!("error" in lines[0]), `${id}: ${lines[0].error} ${lines[0].message}`);
    const { buildErrors, ...queries } = lines[0].data;
    assert.deepStrictEqual(buildErrors, [], `${id}: no group refused`);
    for (const key of [...both, ...direct]) assert.ok(key in queries, `${id} asks ${key}`);
    for (const key of Object.keys(queries)) {
      assert.ok(both.includes(key) || direct.includes(key), `${id}: ${key} is asked and not listed`);
    }
    for (const key of both) {
      assert.deepStrictEqual(Object.keys(queries[key]), ["direct", "all"], `${id} ${key}: both calls`);
    }
    for (const key of direct) {
      assert.deepStrictEqual(Object.keys(queries[key]), ["direct"], `${id} ${key}: the direct call only`);
    }
  }
  // A workbook that holds a value is refused: one line with the error, and nothing is set or calculated in it.
  for (const id of ["M1", "PREC"]) {
    const host = makeHost(false, null, true);
    const lines = [];
    await load(host, lines).__run(id);
    assert.strictEqual(lines.length, 1, `${id} in a used workbook: one line`);
    assert.strictEqual(lines[0].error, "WorkbookNotFresh", `${id} in a used workbook is refused`);
    assert.ok(lines[0].message.includes("Sheet2!A1"), "the refusal names what the workbook holds");
    const touched = host.timeline.filter((e) => e.set || e.call === "context.application.calculate");
    assert.deepStrictEqual(touched, [], `${id}: a refused workbook is left untouched`);
  }
  // In Script Lab: one button a case, the M cases and then the P cases. A click logs its case's line and shows
  // it under the buttons, and turns the buttons off.
  const open = () => {
    const page = fakePage();
    const shown = [];
    const loaded = load(makeHost(false), shown, page);
    return { page, logged: () => shown.map((line) => JSON.stringify(line)), loaded };
  };
  const { loaded } = open();
  await checkButtons(open, [...ids, ...loaded.__p], (line) => line.id);
  assert.deepStrictEqual([...loaded.__p].slice(-2), ["PREC", "DEP"]);
  const each = `${ids.length} cases, each from a host with iteration off and on`;
  const rest = "PREC and DEP; a used workbook refused; a button a case";
  console.log(`ok: ${each}; M20 under ${failures} failed syncs; ${rest}`);
})().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
