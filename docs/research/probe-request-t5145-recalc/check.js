// Checks the sequencing of cases.js without Excel: node check.js
//
// It runs every M case against a recording fake of Office.js and asserts what the cases depend on. It says nothing
// about Excel's values: those are what the request asks for. Checked here:
// - every M case logs exactly one line, with no error;
// - every case's line says whether iterative calculation was on when it started;
// - M20 (a loop closed in Manual mode) turns iterative calculation off before it closes the loop, keeps it off
//   through F9, logs both settings and puts the first one back. It is run from a host with iteration on and from
//   one with it off (review of CR-321738056, round 1: on the desktop the setting is the application's);
// - M20 puts the iteration setting back when one of its syncs fails, whichever one, whether the failed sync had
//   applied its writes or not (round 2 of the same review), and the mode is Automatic afterwards.
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const src = fs.readFileSync(path.join(__dirname, "cases.js"), "utf8");

// A host whose application remembers the calculation mode and the iteration setting; every other property is a
// permissive fake. A write of one of the two settings takes effect at the next sync, as in Office.js. Each set and
// each call is recorded with the two settings as they were at that moment. `fail`, if given, makes the sync with
// that number (from 1) reject once, after applying the queued writes (`applied`) or dropping them.
function makeHost(iterationOn, fail) {
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

async function runCase(id, iterationOn, fail) {
  const host = makeHost(iterationOn, fail);
  const lines = [];
  const sandbox = {
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
  vm.runInContext(`${src}\n;globalThis.__run = run; globalThis.__ids = Object.keys(M);`, sandbox);
  let rejected = null;
  await sandbox.__run(id).catch((e) => {
    rejected = e; // only the sync of run()'s own cleanup can make run() reject
  });
  return { host, lines, ids: sandbox.__ids, rejected };
}

(async () => {
  const { ids } = await runCase("M1", false);
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
  // before it M20's own restore. A failure of the restore itself that drops its write cannot be undone by the
  // script, so that one combination is left out.
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
  const each = `${ids.length} cases, each from a host with iteration off and on`;
  console.log(`ok: ${each}; M20 under ${failures} failed syncs`);
})().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
