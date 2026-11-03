---
id: T-3966
title: "Probe the M6 argument kind no open task lists: an error as TRANSPOSE's array"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TRANSPOSE's `array` has an Excel-backed, passing cell for the one argument kind the report's argument coverage still
misses (an error), so it reaches `covered == expected`.

## Context
- Gap triage 2026-11-03 (opus, after T-3880; `make eval` on 786fc8b2, excel-graph evidence; report.json
  `argument_coverage`, decision 0031; decision 0094). Every failing cell of that run has an open owner, so the
  argument kinds are the evidence gap left. T-2864 (dates) and T-2865 (lookup) cover the other M6 gaps and do not
  name TRANSPOSE.
- Coverage on that run: TRANSPOSE.array (mode `array`) is 10 of 11. It has seen number, zero, negative, fraction,
  numeric text, text, empty text, boolean, blank and range, and is missing **an error**.
- A kind counts only when an Excel-evidence cell passes it as a literal or a single-cell reference. Nested calls
  such as `NA()` are not classified (docs/progress.md "Argument coverage"), so use literal errors and a cell holding
  one.
- An array-mode parameter given a scalar error probably returns that error as a 1x1 result. A range whose cells
  hold errors should transpose each error into place. Neither is probed for TRANSPOSE yet, so record what Excel
  does, including the spill extent.

Write one probe case, probes/m6-argument-kind-gaps-transpose (excel-probe skill, excel-graph, tag M6):
- `=TRANSPOSE(#N/A)`, `=TRANSPOSE(#DIV/0!)`, `=TRANSPOSE(A1)` with A1 `=1/0`, each also as
  `=ERROR.TYPE(TRANSPOSE(..))`.
- `=TRANSPOSE(A1:A3)` over A1 `=1/0`, A2 `2`, A3 `=NA()`, with the three spill cells checked, plus one cell
  checked outside the spill so a wrong extent shows up (decision 0063).
- `=TRANSPOSE({1,#N/A;#DIV/0!,4})`, an array constant holding errors.

## Acceptance criteria
- [ ] probes/m6-argument-kind-gaps-transpose is committed with excel-graph oracle values. After `make eval`,
      report.json `argument_coverage` shows `covered == expected` for TRANSPOSE.array.
- [ ] Each cell where ours differs from Excel is either fixed in this PR (a small fix in TRANSPOSE's own module) or
      listed in a new functions-lookup task (campaign: semantics, with a `## Visible-parity stage` section) filed
      with `make new-task` that names the case and cells. The new known failures are recorded with
      `make eval-baseline` and each one has that owner (decision 0245).
- [ ] Any new Excel rule goes into docs/excel-semantics/ (arrays.md), citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The M6 parameters T-2864 and T-2865 list; the M5 and M7 gaps (T-3965, T-3967).
