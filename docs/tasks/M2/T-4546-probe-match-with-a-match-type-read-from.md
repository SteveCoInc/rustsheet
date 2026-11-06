---
id: T-4546
title: "Probe MATCH with a match_type read from a cell holding numeric text or a boolean"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/lookup.md cites excel-graph cells for MATCH whose match_type is a reference to a cell holding
numeric text (`"1"`, `"-1"`, `" 0"`) or a boolean (TRUE, FALSE). Those forms stop being an unprobed gap in an M2 rule
that the engine already implements.

## Context
- Gap triage 2026-11-06 (`make eval` on main 25779368). MATCH is M2 with catalog prior 5, the highest of any open
  lookup question. lookup.md, "MATCH descending search scans from the top; a text match_type read from a cell is
  #REF!", says under Implemented in: "`fn_match` / `match_type_arg` ... (Rule 2: non-numeric text from a cell only;
  numeric text and booleans in the cell are unprobed)".
- Evidence so far: non-numeric text and every error kind read from a cell give `#REF!` (probes/match-descending-argcheck,
  match-type-text-passthrough, m2-argument-kind-gaps A22 and A25, match-type-error-reference E1:E8). Literal text gives
  `#VALUE!` and a literal error propagates. probes/m2-argument-kind-gaps has a TRUE cell (M3) and a "2" cell (M5), but
  no MATCH reads them.
- What the engine does today (a scratch run on main 25779368, A1:A3 = 1, 2, 3, `MATCH(2,A1:A3,D)`): D = "1" gives 2,
  TRUE gives 2, "-1" gives `#N/A`, " 1" gives 2. So it reads numeric text and booleans from a cell as numbers. If Excel
  gives `#REF!` there, as it does for other text, that is a semantics fix for `match_type_arg`.
- Probe (excel-probe skill, excel-graph, tag M2), on lookup_array A1:A5 = 10, 20, 30, 40, 50 and its descending copy.
  Use lookup values 30 and 35 so that exact, approximate and descending answers differ. Give match_type as a reference
  to a cell holding each of: "1", "0", "-1", " 1", "1E0", "2" (numeric text out of range), TRUE, FALSE, and the
  formula cells `="1"` and `=TRUE`. Add the same values as literals for comparison ("1", TRUE), and the same reference
  picked by IF (`IF(TRUE,D1)`). Read each error through ERROR.TYPE.
- Run the case. Any cell where ours differs from Excel goes to a functions-lookup task filed with `make new-task`
  (`campaign: semantics`, with a `## Visible-parity stage` section) that names the case and cells.

## Acceptance criteria
- [ ] probes/t4546-match-type-cell-numeric-text-boolean is committed with excel-graph oracle values and covers every
      form in Context.
- [ ] lookup.md: the "numeric text and booleans in the cell are unprobed" note is replaced by the evidenced rule, with
      the cells cited.
- [ ] Each cell where ours differs is recorded with `make eval-baseline` and owned by the functions-lookup task filed
      for it, named in the Outcome; no other new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine fix itself (the filed functions-lookup task does it).
- VLOOKUP/HLOOKUP range_lookup read from a cell (m2-argument-kind-gaps covers the error and text forms).
