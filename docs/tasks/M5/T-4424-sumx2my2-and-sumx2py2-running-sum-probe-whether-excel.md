---
id: T-4424
title: "SUMX2MY2 overflow order: probe whether Excel adds x^2 and then -y^2 to its running sum, or keeps an overflowing square in 80 bits, where the pair term is finite"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
SUMX2MY2 is `#NUM!` exactly where Excel's is, at pairs whose x^2 or running sum + x^2 overflows a double while the
pair term x^2 - y^2 and the final sum are finite. Today this follows the engine's one-term-per-pair fold, and no Excel
cell decides it.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 20b163db, excel-graph sidecars). `make unowned-off ALL=1` gives 0
  unowned visibly-off cells, so this triage read the rules the semantics docs mark unprobed.
- numbers.md "SUMPRODUCT, SUMX2MY2 and SUMX2PY2 overflow at a running sum" (probes/t3677-sumx-running-overflow-snap,
  excel-graph, 2026-11-03, T-3677) cites:
  - A4:A6: SUMX2MY2 adds one x^2 - y^2 per pair, not SUMX2 - SUMY2. The pairs (1.2E+154, 0), (1.2E+154, 0),
    (0, 1.2E+154) are `#NUM!`, the reordered pairs give 1.4400000000000002E+308, and two (1.2E+154, 1.2E+154) pairs
    give 0.
  - A7 and A20: SUMX2PY2's term overflows.
  - A19: SUMXMY2 over ±1E+154 is `#NUM!`.
- It ends: "Unprobed: whether SUMX2MY2's running sum takes x^2 and then -y^2 (which could overflow where the term does
  not)". No A4:A6 pair tells these three models apart:
  - (a) one double term per pair, `s + (x*x - y*y)` (ours);
  - (b) `(s + x*x) - y*y`;
  - (c) the term or the sum kept in an 80-bit x87 register, so an x^2 above DBL_MAX does not overflow until it is stored.
- Ours: `fn_sumx2my2` in crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs folds
  `flush_subnormal(x * x) - flush_subnormal(y * y)` per pair (model a). A scratch case on a release build of 20b163db
  gives:
  - `SUMX2MY2({1.3E+154,1.2E+154},{0,1.2E+154})` 1.6899999999999998E+308 (model b would be `#NUM!`);
  - `SUMX2MY2({1.2E+154,1.3E+154},{1.2E+154,0})` the same.
- Excel evidence: none yet beyond A4:A6. The probe below decides between models a, b and c by error against number, so
  no digits are fitted.

## Probe
excel-probe skill, excel-graph, tag M5. Name the case probes/t4424-sumx2my2-overflow-order. Use literal arrays and
one-row ranges holding the same numbers, with an `ERROR.TYPE` companion for each cell.
- Running sum then a cancelling pair: `{1.3E+154,1.2E+154}` against `{0,1.2E+154}`, the reverse order, and
  `{1.2E+154,1.3E+154}` against `{1.2E+154,0}`. Model b is `#NUM!` here, a and c give 1.69E+308.
- A square that overflows alone: `SUMX2MY2({1.5E+154},{1.4E+154})`, where x^2 = 2.25E+308 and the term is 2.9E+307, and
  `SUMX2MY2({1.4E+154},{1.5E+154})`. Model c answers both, a and b give `#NUM!`.
- A negative running sum: `{0,1.2E+154}` against `{1.3E+154,1.2E+154}`, and one more cell with -y^2 overflowing alone.
- Controls: the same pairs through SUMXMY2 and SUMX2PY2.

## Acceptance criteria
- [x] probes/t4424-sumx2my2-overflow-order is committed with excel-graph oracle values.
- [x] Every cell passes. If Excel follows model b or c, the closure in `fn_sumx2my2` (and SUMX2PY2's, if its cells
      differ) changes to that order, with a unit test per model-deciding cell. If no single model explains every cell,
      or the fix would change any digit of a passing SUMX2MY2, SUMX2PY2 or SUMXMY2 cell, hand the task up (work-loop
      step 5b): that is an operation-order fit.
- [x] numbers.md's "Unprobed: whether SUMX2MY2's running sum takes x^2 and then -y^2" is replaced by the result, citing
      the cells.
- [x] `make eval-baseline` records no regression; `rustsheet-eval accuracy` passes; `make check` green; reviewers approve.

## Out of scope
- SUMXMY2's last digit on a 1.5E+16 sum (T-2384).
- SUMPRODUCT's running overflow and snap (T-3677, done).

## Handoff (resumed)
Probe done and evidenced (excel-graph): Excel follows model (a), one double term per pair, as `fn_sumx2my2` already does (rows 1-7 SUMX2MY2: 1.69E+308, 1.69E+308, -1.69E+308, -1.69E+308, #NUM!, #NUM!, #NUM!). No engine change needed. numbers.md is updated and the case + oracle sidecar are committed.
Missing: `probes/t4424-sumx2my2-overflow-order.math.json`. `cd tools && uv run rustsheet-accuracy math --suite probes` has no per-case option, recomputes the Bessel cases (m9-bessel-order-limits, t2546, t2881) with mpmath and was killed at the 30 min background limit twice without writing the new sidecar. Next: run it with `--workers` on a bigger box or in a dedicated long session, commit only the t4424 sidecar (restore the other .math.json files), then `--check`, `make check-fast`, mark done with the Outcome (model a confirmed; no code change), review, PR.

## Outcome
- **Probe:** probes/t4424-sumx2my2-overflow-order, excel-graph, 84 cells. The engine passes all 84 with no code change
  (`rustsheet-eval show`: checked 84, passed 84).
- **Result:** Excel follows model (a), one finished double term x^2 - y^2 per pair, which is what `fn_sumx2my2`
  already does.
  - A1:A2 are 1.6899999999999998E+308 in either order, which rules out model b.
  - A3:A4 are the negative twin.
  - A5:A7 are `#NUM!`, where x^2 or -y^2 overflows alone while the term is finite. That rules out an 80-bit square
    (model c).
- **Docs:** numbers.md's "Unprobed" line is replaced by this result, with the cells cited.
- **Math sidecar:** the handoff's blocker was that `rustsheet-accuracy math --suite probes` recomputes every case.
  The resumed run instead wrote this case's sidecar with `eval_math.case_math` on the one case. It has 42 math
  values. `eval_math.check_suite('probes')` reports no problems.
- **Eval delta:** one new case of 84 cells, all passing. No baseline change, and no regression.
