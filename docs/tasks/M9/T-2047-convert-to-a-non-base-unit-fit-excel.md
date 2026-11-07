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
- T-4630 (2026-11-06): distance factors in angstroms (m = 1E10) fix every distance cell below (m to ft, in to mi, the
  light-year and parsec sweep cells); what is left here is the area and other classes' non-base cells
  (`"m2"` to `"ly2"`, `ga`, `yd` to `yd`).
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
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
- Gap triage 2026-11-03 (origin/main eda15c023, excel-graph): the only failing cells of the whole report that no open
  task named. They are T-3861's (done) residue, and engineering.md "CONVERT constants and evaluation order" puts them
  here ("seven of the eight misses have a light-year or parsec factor (T-2047's digits)"). Every one is 1 or 2 ulp, so
  none is visibly off, and each converts through a light-year (`ly`), parsec or ångström (`ang`) factor with a prefix:
  - probes-numeric/t3861-convert-prefix-holdout.case.json (2): Calc!A34, A63
  - probes-numeric/t3861-convert-prefix-sweep.case.json (7): Calc!A6, A56, A70, A85, A89, A106, A109
  Examples: holdout A63 `CONVERT(1,"mly","hang")` is 9.460730472580801E+20 in Excel and 9.4607304725808E+20 here.
  Sweep A109 `CONVERT(0.00041,"parsec","Pm")` is 0.01265127808325437 in Excel and 0.012651278083254369 here.
  Both cases were T-3861's hold-out and fresh sweep. If they are used to fit here, they are training data: score the
  fit on a new sweep (R4.2).
- Engine: `fn_convert` and the unit table in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [ ] The three known-failure lines (probes/scalar-lift-open-a Sheet1!Z8, Z9, Z10) leave eval/baseline/probes.txt, and
      gen-m9/932b33b1ef8b7608 Calc!A14 leaves eval/baseline/gen-m9.txt,
      with no new lines in any suite.
- [ ] The T-2046 sweep matches >= 99.9% bit for bit (or the misses are listed in engineering.md and
      filed), and CONVERT.xlsx and probes/convert-targets still pass in full.
- [ ] engineering.md's CONVERT entry states the fitted rule with the sweep cells.
- [ ] The nine t3861 cells listed above pass bit-exact, or the Outcome lists each one left with its unit pair.
- [ ] R4 guards of reviewer-excel-fidelity check 7 (decision 0188), in the Outcome. Hold-out counts: train against
      hold-out (bit-exact, within 4 ulp, within 15 digits) for the chosen form and the correctly rounded value, margin
      stated before scoring. A fresh sweep, oracle-filled after the form is frozen. The Gate A envelope result on it.
      `FITTED(T-2047, <n> cells, <range>)` on every fitted factor or cut, applied only inside that range. No fit below
      the 4-ulp reproducibility floor without a desktop-backed check or an owner ruling (R4.4). No keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Temperature conversions; CONVERT error order (T-1385).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
