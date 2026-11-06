---
id: T-4538
title: "INDIRECT's a1 read from a cell or omitted: probe FALSE/TRUE text, booleans, numbers, blank and empty-text cells and INDIRECT(ref,), and match"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDIRECT's `a1` argument has Excel evidence for the forms docs/excel-semantics/lookup.md "INDIRECT's a1 argument is
coerced like a boolean" lists as not probed (text "FALSE" and booleans read from cells), plus a blank cell, an
empty-text cell and an omitted a1 (`INDIRECT("H2",)`), and ours matches every one of them.

## Context
- Gap triage 2026-11-06 (opus; `make eval` on 257793681, excel-graph evidence). Every failing cell of that run that
  calls a functions-lookup or functions-criteria function is either owned by an open task (T-4428 PIVOTBY, T-2686
  GROUPBY operators, T-2425 FORECAST.ETS, T-2600/T-2607 MAKEARRAY) or is blamed on another function nested in it, so
  the lane's remaining gaps are unprobed rules (decision 0031).
- The rule as recorded (lookup.md, evidence probes/indirect-text-broadcast B3:B10, excel-graph 2026-10-05, T-0957):
  a nonzero number is TRUE (`INDIRECT("H2",2)`, `INDIRECT("H2",-1)` = 5), `"TRUE"` text is TRUE, `0` is FALSE (R1C1,
  so `INDIRECT("H2",0)` is `#REF!` and `INDIRECT("R2C8",0)` is 5), `""` and `"x"` are `#VALUE!`. "Not probed: `"FALSE"`
  text and booleans read from cells."
- Ours: `indirect_of` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs) casts a1 with
  `cast_to_bool`; a blank cell and an omitted argument (`CalcResult::EmptyArg`/`EmptyCell`) take whatever that cast
  gives, which no Excel cell checks today. Excel's omitted optional arguments often read as FALSE/0 rather than the
  default (a reason `INDIRECT("H2",)` may well be `#REF!`), so do not assume.

## Probe
One case, probes/t4538-indirect-a1-cells (excel-probe skill, excel-graph, tag M6). H2 = 5 so that A1 style reads
5 and R1C1 style reads `#REF!` for "H2"; also ask each row with "R2C8" so both styles are visible. a1 forms:
- a literal `"FALSE"`, `"false"`, `"True"`, `" TRUE"` (leading space);
- cells holding TRUE, FALSE, 0, 1, -0.5, 0.5, the text "FALSE", the text "1" (numeric text), `=""`, a blank cell,
  and `=NA()` (also wrapped in `ERROR.TYPE`);
- an omitted a1: `INDIRECT("H2",)` and `INDIRECT("R2C8",)`;
- a one-cell range `B1:B1` holding FALSE.
Write the predictions (ours, from a scratch build) into docs/research/t4538 before the fill, as other probe tasks do.

## Acceptance criteria
- [ ] probes/t4538-indirect-a1-cells is committed with excel-graph oracle values (the questions above, each in both
      reference styles).
- [ ] Every cell of it passes; any rule ours did not already follow is fixed in `indirect_of` (or the shared
      coercion helper it calls, extended rather than copied) with a unit test citing the probe cells.
- [ ] docs/excel-semantics/lookup.md "INDIRECT's a1 argument is coerced like a boolean" lists the new forms with
      their cells and drops the "Not probed" line it answers.
- [ ] No new known-failure lines in any eval/baseline/*.txt; `make check-fast` green, CI green; reviewers approve.

## Out of scope
- The array-lift of a1 (`INDIRECT("B1",{1})`: T-2577, done) and ref_text kinds (T-2865).
