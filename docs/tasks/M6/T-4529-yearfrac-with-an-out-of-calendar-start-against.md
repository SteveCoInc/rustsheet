---
id: T-4529
title: "YEARFRAC with an out-of-calendar start against a text, boolean, empty or multi-cell-range end (and the reverse): probe which check wins and match"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
YEARFRAC's order between the start date's calendar check and the end date's coercion (text,
boolean, empty, a multi-cell range) has Excel evidence, and the engine follows it, so the UNVERIFIED
sentence of docs/excel-semantics/dates.md "YEARFRAC reads its start's range before the end's error
(T-2266)" is settled.

## Context
- VERIFIED today: an out-of-calendar start beats an error end (gen-m6/3516b9fbbd60269b Calc!A301,
  gen-m6/d0c30225243fde60 Calc!A266, `#NUM!`), and between two errors the start's wins
  (probes/scalar-lift-open-d W2, W5). Basis is read and range-checked first
  (probes/yearfrac-basis-order).
- UNVERIFIED, no open owner: an out-of-calendar start (`-1`, `2958466`, `1E+15`) against an end that
  is non-numeric text (`"x"`), numeric text (`"45000"`, which coerces), TRUE (YEARFRAC refuses
  booleans: `cast_number_no_bools_ref`), an empty cell, or a multi-cell range (refused per element
  by the `LiftArrays` lift, probes/scalar-lift-shapes H6). Ours gives `#NUM!` for all of them,
  because `yearfrac_of` (crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs) checks the
  start's calendar range before it reads the end at all. The reverse (a text or boolean start
  against an out-of-calendar end) is `#VALUE!` in ours and is unprobed too.
- Opus: Excel may read both dates before either range check (as the two-error cells allow), which
  would flip several of these to `#VALUE!`; the probe has to separate "start fully first" from
  "coerce both, then range-check" and from "per-argument kind" readings, and the fix may move the
  check across the lift.
- Not T-4454 (serials just below midnight) nor T-2591/T-1954 (WORKDAY/NETWORKDAYS argument edges).
- Use the excel-probe skill (excel-graph oracle).

## Acceptance criteria
- [ ] A committed probe case with an excel-graph sidecar holds YEARFRAC with start in {-1,
      2958466, 1E+15} against end in {"x", "45000", TRUE, FALSE, an empty cell, a cell holding
      "x", a two-cell range, a 2x2 range, `{"x"}`}, each with basis omitted and basis 1, plus the
      reverse pairs (start text, boolean, empty, range; end out of calendar).
- [ ] Every new cell passes, or each remaining failure is owned by a filed task named in dates.md.
- [ ] dates.md's YEARFRAC entry replaces its UNVERIFIED sentence with the probed rule and cites
      the cells; if the rule differs from "start's range first", it says what was wrong.
- [ ] No new known-failure lines in eval/baseline/gen-m6.txt or eval/baseline/probes.txt other
      than the new case's owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The day-count arithmetic of each basis.
- Other Analysis ToolPak date functions (file a follow-up if the probe suggests a shared rule).
