---
id: T-3967
title: "Probe the M7 argument kinds no open task lists: an error as BYROW's and BYCOL's array"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BYROW's and BYCOL's `array` each have an Excel-backed, passing cell for the one argument kind the report's argument
coverage still misses (an error), so both reach `covered == expected`.

## Context
- Gap triage 2026-11-03 (opus, after T-3880; `make eval` on 786fc8b2, excel-graph evidence; report.json
  `argument_coverage`, decision 0031; decision 0094). Every failing cell of that run has an open owner, so the
  argument kinds are the evidence gap left. T-2873 to T-2878, T-3513 and T-3514 cover the other M7 gaps. T-2878
  covers LAMBDA, LET, REDUCE, SCAN and ISOMITTED but not BYROW or BYCOL.
- Coverage on that run: BYROW.array and BYCOL.array (mode `array`) are each 10 of 11. Each has seen number, zero,
  negative, fraction, numeric text, text, empty text, boolean, blank and range, and is missing **an error**. Their
  `function` parameter (mode `lambda`) expects no kinds.
- A kind counts only when an Excel-evidence cell passes it as a literal or a single-cell reference. Nested calls
  such as `NA()` are not classified (docs/progress.md "Argument coverage"), so use literal errors and a cell holding
  one.
- The open question is whether Excel hands the error to the LAMBDA as the row or column (so `LAMBDA(r,ISERROR(r))`
  gives TRUE and `LAMBDA(r,1)` gives 1) or returns the error before calling it. Write rows that tell the two apart.

Write one probe case, probes/m7-argument-kind-gaps-byrow-bycol (excel-probe skill, excel-graph, tag M7), for both
BYROW and BYCOL:
- `=BYROW(#N/A,LAMBDA(r,1))`, `=BYROW(#N/A,LAMBDA(r,ISERROR(r)))`, `=BYROW(#DIV/0!,LAMBDA(r,SUM(r)))`, and the same
  with the error in a cell (`=BYROW(A1,LAMBDA(r,1))` with A1 `=1/0`). Wrap the error-valued results in `ERROR.TYPE`.
- A multi-row range with one error row, `=BYROW(A1:B3,LAMBDA(r,ISERROR(SUM(r))))`, with every spill cell checked and
  one cell outside the spill (decision 0063).

## Acceptance criteria
- [ ] probes/m7-argument-kind-gaps-byrow-bycol is committed with excel-graph oracle values. After `make eval`,
      report.json `argument_coverage` shows `covered == expected` for BYROW.array and BYCOL.array.
- [ ] Each cell where ours differs from Excel is either fixed in this PR (a small fix in the BYROW/BYCOL module) or
      listed in a new functions-logic task (campaign: semantics, with a `## Visible-parity stage` section) filed with
      `make new-task` that names the case and cells. The new known failures are recorded with `make eval-baseline`
      and each one has that owner (decision 0245).
- [ ] Any new Excel rule goes into docs/excel-semantics/arrays.md (the LAMBDA helpers' entry), citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The M7 parameters T-2873 to T-2878, T-3513 and T-3514 list; the M5 and M6 gaps (T-3965, T-3966).
