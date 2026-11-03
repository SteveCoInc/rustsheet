---
id: T-2047
title: "CONVERT to a non-base unit: fit Excel's last digit (m to ft is 3.2808398950131235)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2046]
defer: phase-1b
---

## Goal
CONVERT matches Excel bit for bit when converting from a base unit to a non-base unit, starting with
`CONVERT(1,"m","ft")` = 3.2808398950131235, from a factor table or order fitted on the T-2046 sweep.

## Context
Needs an oracle sweep: T-2046 (only two m→ft points exist).
- Known failures (gap triage 2026-10-19): probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10
  `CONVERT(1,"m","ft")`: Excel 3.2808398950131235, ours 3.280839895013123.
- The current rule (docs/excel-semantics/engineering.md "CONVERT constants and evaluation order":
  `value * F_from / F_to`, prefixes after) fits all 509 CONVERT.xlsx cells, which only go unit → base.
  A reciprocal factor for ft (K = 3.2808398950131235; m→ft `v * K`, ft→m `v / K`) fits the four
  known ft cells (m→ft at 1 and 2.5, ft→m at 1 and 4); x / 0.0254 / 12 fits the two m→ft cells but
  not ft→m. Decide per unit or per class with the sweep, and keep every currently passing
  CONVERT cell passing.
- T-3817 probe (excel-graph), one ulp off: probes/convert-unit-names Sheet1!B33 `CONVERT(1,"m","ft")`,
  B961 and B969 (`"m2"` to `"ly2"`, `"ly^2"`).
- T-2676 sweep (gen-m9 round 2, gap triage 2026-10-26, excel-graph), not visibly off: gen-m9/932b33b1ef8b7608 Calc!A14
  `=CONVERT((0--(-0.0001)),"in","mi")` (the number is -0.0001): Excel -1.5782828282828283E-9, ours -1.578282828282828E-9
  (one ulp). A non-base to non-base pair (in to mi), so the same factor question as m to ft; add in and mi to the
  T-2046 sweep's units if they are not there.
- Gap triage 2026-10-30, same m to ft value, no open task named them: probes/eng-convert-boolean-unit Sheet1!A7
  `=CONVERT(1,{"m",TRUE},"ft")`, A29 (spilled from A28 `=CONVERT(1,{TRUE;"m"},"ft")`) and B8 (spilled from A8
  `=CONVERT(1,"m",{TRUE,"ft"})`): Excel 3.2808398950131235, ours 3.280839895013123.
- Gap triage 2026-11-03 (`make eval` on origin/main b6ab50e5, excel-graph; no open task named them): the light-year
  and parsec misses T-3861's sweep and hold-out left to this task's factor digits (engineering.md "CONVERT constants
  and evaluation order", "Open gaps"), all 1 or 2 ulp, within 15 digits. T-2046's sweep covers `ly` and `parsec`, so
  fit them with m to ft. Examples: probes-numeric/t3861-convert-prefix-sweep Calc!A109 `CONVERT(0.00041,"parsec","Pm")`
  Excel 0.01265127808325437, ours 0.012651278083254369 (misses under every form T-3861 scored); Calc!A6
  `CONVERT(0.017,"dang","Ely")` Excel 1.7969014178418463E-47, ours 1.7969014178418466E-47;
  probes-numeric/t3861-convert-prefix-holdout Calc!A63 `CONVERT(1,"mly","hang")` Excel 9.460730472580801E+20,
  ours 9.4607304725808E+20. Cells:
  - probes-numeric/t3861-convert-prefix-sweep.case.json (7): Calc!A6, A56, A70, A85, A89, A106, A109
  - probes-numeric/t3861-convert-prefix-holdout.case.json (2): Calc!A34, A63
- Engine: `fn_convert` and the unit table in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [ ] The three known-failure lines (probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10) leave eval/baseline/probes.txt, and
      gen-m9/932b33b1ef8b7608 Calc!A14 leaves eval/baseline/gen-m9.txt,
      with no new lines in any suite.
- [ ] The T-2046 sweep matches >= 99.9% bit for bit (or the misses are listed in engineering.md and
      filed), and CONVERT.xlsx and probes/convert-targets still pass in full.
- [ ] The 9 t3861 light-year and parsec cells above pass bit-exact, or each miss is listed in engineering.md and
      filed (probes-numeric is ungated, so they have no baseline line; `make eval-show CASE=...`).
- [ ] engineering.md's CONVERT entry states the fitted rule with the sweep cells.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), in the Outcome: hold-out counts (train and
      hold-out side by side, bit-exact / within 4 ulp / within 15 digits, for the chosen factor table or order and
      the correctly rounded conversion, from a split stated before scoring); a fresh sweep generated and
      oracle-filled after the fit is frozen; its Gate A envelope result with every point outside listed; a
      `FITTED(T-2047, n_cells, range)` tag and range on each fitted constant (a structural order choice owes no tag);
      no target below the reproducibility floor; no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Temperature conversions; CONVERT error order (T-1385).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
