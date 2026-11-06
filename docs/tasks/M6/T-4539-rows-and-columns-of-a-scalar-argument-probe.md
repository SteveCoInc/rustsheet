---
id: T-4539
title: "ROWS and COLUMNS of a scalar argument: probe empty text, a text cell and an error cell (ROWS.array empty_text coverage gap), and match"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ROWS's `array` parameter reaches `covered == expected` in report.json `argument_coverage` (it is 10 of 11, missing
**empty text**), and ROWS and COLUMNS of every scalar argument kind have paired Excel cells that ours matches.

## Context
- Gap triage 2026-11-06 (opus; `make eval` on 257793681, excel-graph evidence; report.json `argument_coverage`,
  decision 0031). ROWS.array has seen number, zero, negative, fraction, numeric text, text, boolean, error, blank and
  range; empty text has no Excel cell. COLUMNS.array is 11 of 11: `COLUMNS("")` appears only nested, in gen-m9
  b4173fe25109a238 Calc!A286 (`ODDLYIELD(..,COLUMNS(""),..)`) and 4250f880ab640c49 Calc!A263, which pass.
- No open task lists ROWS: T-3966 is TRANSPOSE's error kind only, T-2865 leaves array-mode parameters out, T-2873
  covers the M7 reshaping functions, T-3087 the empty value (a blank read through CELL/IFERROR) in COLUMNS among others.
- Ours: `fn_rows`/`fn_columns` call `rows_columns_extent`
  (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs). Whether a scalar error argument is 1 or the
  error, and whether a literal and a cell holding it agree, is exactly what a direct, un-nested probe settles; the
  nested gen-m9 cells only constrain the value through another function.

## Probe
One case, probes/t4539-rows-columns-scalar-kinds (excel-probe skill, excel-graph, tag M6). For each of ROWS and
COLUMNS, called directly (not nested), each argument as a literal and as a single-cell reference to a cell holding it:
`""` (and a formula cell `=""`), "x", "1", 0, -1, 0.5, TRUE, `#DIV/0!` (literal, `1/0` cell, `NA()` call), a blank
cell. Wrap every error-valued cell in `ERROR.TYPE` too. Record ours in docs/research/t4539 before the fill.

## Acceptance criteria
- [ ] probes/t4539-rows-columns-scalar-kinds is committed with excel-graph oracle values; after `make eval`,
      report.json `argument_coverage` shows `covered == expected` (11 of 11) for ROWS.array.
- [ ] Every cell of it passes; a rule ours did not follow is fixed in `rows_columns_extent` with a unit test citing
      the probe cells.
- [ ] Any rule not already in docs/excel-semantics/lookup.md is recorded there citing the cells.
- [ ] No new known-failure lines in any eval/baseline/*.txt; `make check-fast` green, CI green; reviewers approve.

## Out of scope
- ROWS/COLUMNS of arrays of references or lifted calls (T-2577, T-2436 cells), and the empty value of T-3087.
