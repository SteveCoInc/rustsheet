---
id: T-4396
title: "AVERAGEIFS and MINIFS over a lifted OFFSET: probe, then spill per element like MAXIFS and SUMIFS (ours #VALUE!)"
milestone: M9
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
AVERAGEIFS and MINIFS whose ranges are an array of references (a lifted OFFSET, SPEC-0014) give Excel's result, backed
by excel-graph cells. Today they are the only two conditional aggregates that read the value view there: the engine
returns a single `#VALUE!` where MAXIFS, SUMIFS, COUNTIFS and AVERAGEIF spill one result per element.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 8465f6f1, excel-graph sidecars; `scripts/exit_owners.py --all-off`
  and `--milestone M5..M11`): every failing cell has an open owner (the 40 FORECAST.ETS web-`#N/A` cells now with
  T-4207), so this triage looked at rules the semantics docs still mark unprobed. docs/excel-semantics/lookup.md says
  twice that AVERAGEIFS and MINIFS over a lifted OFFSET are unprobed (section "OFFSET, INDIRECT and TRIMRANGE lift
  into arrays of references": "AVERAGEIFS and MINIFS are unprobed"; section "Statistical aggregates over a lifted
  OFFSET": "AVERAGEIFS and MINIFS are unprobed and read the value view"). T-2833 (todo, research) probes only partly
  lifted COUNTIFS, AVERAGEIF and MAXIFS forms, not these two functions.
- Sibling evidence (excel-graph, 2026-10-26, T-2065; data A1:A3 = 1, 100, 10000, B1:B3 = 2, 200, 20000, A4 "5",
  A6 TRUE, A7 7):
  - probes/t2065-lift-maxifs E1:E2: `MAXIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")` spills 1, 100.
  - probes/t2065-lift-countifs E1:E2: `COUNTIFS(OFFSET(A1,{0;1},0),">0")` spills 1, 1.
  - probes/t2065-lift-averageif E1:E2: `AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")` spills 1, 50.5.
- Ours on the same data (rustsheet-eval on a scratch case, release build of 8465f6f1):
  - `AVERAGEIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")`: `#VALUE!`, no spill.
  - `MINIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")`: `#VALUE!`, no spill.
  - `AVERAGEIFS(OFFSET(A1,0,0,{1;2}),A1:A2,">0")`: `#VALUE!`.
  - The MAXIFS and SUMIFS forms spill 1, 100.
- The likely rule, by analogy and UNVERIFIED until the probe: AVERAGEIFS lifts like SUMIFS (average_range and every
  criteria range) and MINIFS like MAXIFS (min_range and every criteria range). The probe decides; Excel is right even
  if it differs from its siblings.
- Code: `Function::per_reference_element` in
  crates/rustsheet-engine/src/functions/lookup_and_reference/ref_lift.rs lists the per-element callers and the
  argument positions that take a reference (`Function::Sumifs | Function::Maxifs => index == 0 || index % 2 == 1`, with
  the `per_pair` cost). AVERAGEIFS and MINIFS are missing from it. The work bound and its DEVIATION are those of
  COUNTIF/SUMIFS (lookup.md). SPEC-0014 (docs/specs/SPEC-0014-reference-array-results.md) is the design.

Probe (excel-probe skill, excel-graph, tag M9; one formula per workbook, as T-2065 did, because Excel for the web
refused some combined workbooks), on probes/reference-lift-consumers-b2's data:
- `AVERAGEIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")` and `MINIFS(..)` of the same shape;
- the first range lifted only: `AVERAGEIFS(OFFSET(A1,{0;1},0),A1:A2,">0")`, `MINIFS(OFFSET(A1,{0;1},0),A1:A2,">0")`;
- a criteria range lifted only: `AVERAGEIFS(A1,OFFSET(A1,{0;1},0),">0")`, `MINIFS(A1,OFFSET(A1,{0;1},0),">0")`;
- a height lift: `AVERAGEIFS(OFFSET(A1,0,0,{1;2}),OFFSET(A1,0,0,{1;2}),">0")` and the MINIFS form;
- a second criteria pair, one of its ranges lifted: `AVERAGEIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0",OFFSET(B1,{0;1},0),">0")`.
Name the cases probes/t4396-lift-averageifs-* and -minifs-*.

## Acceptance criteria
- [x] The probe cases above are committed with excel-graph oracle sidecars.
- [x] AVERAGEIFS and MINIFS match every probe cell: per element where Excel lifts (an entry in
      `per_reference_element` with the probed positions and a per-pair cost), and the value view where it does not.
      A unit test in ref_lift.rs or if_ifs.rs pins one lifted call of each.
- [x] No cell of probes/t2065-lift-* or probes/reference-lift-consumers* changes, and `make eval-baseline` records no
      regression.
- [x] lookup.md's two "AVERAGEIFS and MINIFS are unprobed" sentences are replaced by the rule with the probe cells
      cited. The positions that are still inferred stay marked UNVERIFIED.
- [x] `make check` green; reviewers approve.

## Out of scope
- The partly lifted COUNTIFS, AVERAGEIF and MAXIFS forms (T-2833).
- The D-functions and AGGREGATE over a lifted OFFSET.

## Outcome
- Probed (excel-graph, 2026-11-05) nine one-formula workbooks, probes/t4396-lift-averageifs-{both,first,criteria,height,pair2}
  and -minifs-{both,first,criteria,height}. AVERAGEIFS and MINIFS lift per element exactly like SUMIFS and MAXIFS: the
  first range and the criteria ranges. A lifted first range against a two-cell criteria range spills `#VALUE!` per element.
- `Function::per_reference_element` (ref_lift.rs) now lists AVERAGEIFS and MINIFS with SUMIFS and MAXIFS (first range and
  every criteria range, one walk per criteria pair). Static analysis gives them the lifted shape
  (`criteria_and_grid_static_result`). Unit test: `fn_offset_lifted_into_averageifs_minifs` (test_fn_offset.rs), which
  also covers the visit bound.
- lookup.md: both "unprobed" sentences are replaced by the rule, with cells cited. A later criteria range lifted alone,
  and MINIFS with two pairs, stay UNVERIFIED.
- Eval delta: +18 passing cells, all of them new probe cells (9 cases × E1:E2). No regression. probes/t2065-lift-* and
  probes/reference-lift-consumers* are unchanged.
