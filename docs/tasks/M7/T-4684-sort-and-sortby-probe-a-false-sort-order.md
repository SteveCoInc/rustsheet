---
id: T-4684
title: "SORT and SORTBY: probe a FALSE sort_order before a later error and an error cell in a range sort_index, and match"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells settle two SORT/SORTBY argument-order rules that docs/excel-semantics/arrays.md marks UNVERIFIED. The
first is whether a FALSE sort_order is an immediate `#VALUE!` or a validation error that a later argument's error
beats. The second is how an error cell inside a range sort_index is read. arrays.md records the answer and the engine
matches every probe cell.

## Context
- Gap triage 2026-11-06 (evening steward run, `make eval` on agent/steward/2026-11-06). Every failing cell of the run
  in the functions-lookup lane has an open owner (T-4569/T-4570, T-4373, T-2906), and no lookup task was ready. So the
  lane's UNVERIFIED semantics notes are the gap left. SORT and SORTBY are M7 (catalog prior 1.5 and 0.5).
- arrays.md "SORT and SORTBY: which of several bad arguments wins (T-4479)" says FALSE alone is `#VALUE!`
  (`SORTBY(A1:A3,C1:C3,FALSE)`, probes/t4479-sort-bad-argument-order H20). Then: "a FALSE order before a later error is
  UNVERIFIED (no cell has one)". The same entry shows a blank or out-of-range order failing only in validation, so a
  later `NA()` wins, while text or an error order fails at once.
- arrays.md "Dynamic-array and LAMBDA function argument modes" (the sort_index bullet, probes/sort-index-key-ties,
  T-1837) says "an error cell inside a range list is read as an error element by analogy, UNVERIFIED". The same
  bullet has error *elements* of an array list as validation errors, where a later argument's error wins
  (`SORT(..,IF({TRUE;FALSE},1,NA()),1/0)` is `#DIV/0!`, A8). A blank cell in a range list is VERIFIED as `#VALUE!` (A9).
- Related evidence: arrays.md "TOCOL, TOROW and SORT: which bad argument wins (T-3497)" has a FALSE *sort_index* losing
  to a later error (gen-m7). It has no FALSE *sort_order* and no SORTBY. It also says a blank sort_index or sort_order
  "wins with `#VALUE!`", while the T-4479 entry has a blank SORTBY order losing to a later `NA()`. If the new cells
  bear on that difference, say so in both entries.
- No suite has either form. A search of every `eval/suites/*/cases/*.case.json` on 2026-11-06 found no
  `SORTBY(..,FALSE,<later pair>)` and no `SORT(<array>,<range holding an error>)`.
- What the engine does today (scratch evaluation, A1:A3 = 3,1,2, B1:B3 = 9,8,7, C1:C3 = c,a,b):
  - `SORTBY(A1:A3,C1:C3,FALSE,C1:C3,NA())` is `#N/A`, `SORTBY(A1:A3,C1:C3,FALSE,C1:C3,1/0)` `#DIV/0!`,
    `SORTBY(A1:A3,C1:C3,FALSE,NA(),1)` `#N/A`, `SORTBY(A1:A3,C1:C3,FALSE,C1:C2,1)` `#VALUE!`, and
    `SORTBY(A1:A3,C1:C3,1,C1:C3,FALSE)` `#VALUE!`. So the engine treats FALSE as a validation error.
  - `SORT(A1:A3,1,FALSE)` is `#VALUE!` and `SORT(A1:A3,1,FALSE,NA())` is `#N/A`.
  - With D1 = 1 and D2 `=NA()`: `SORT(A1:B3,D1:D2)` is `#VALUE!` and `SORT(A1:B3,D1:D2,1/0)` `#DIV/0!`. With
    E1 `=1/0` and E2 = 1: `SORT(A1:B3,E1:E2)` is `#VALUE!` and `SORT(A1:B3,E1:E2,{1;1},NA())` `#N/A`. So an error cell
    in a range list acts like an error element: a validation `#VALUE!` that a later error beats.
- Code: `fn_sort` and `fn_sortby` in crates/rustsheet-engine/src/functions/spill_functions.rs.
- Probe design: reuse the layout of probes/t4479-sort-bad-argument-order (column G holds the call as text, column H
  holds `IFERROR(TEXTJOIN(",",,<call>),ERROR.TYPE(<call>))`). Wrap each call so it is one cell: an unwrapped
  `ERROR.TYPE(<array>)` fallback spills into the next row and turns into `#SPILL!`. Include the forms above, plus SORT
  with FALSE as sort_order before a later by_col error, and a range sort_index whose error cell comes first
  (`E1:E2`) and last (`D1:D2`). Add one passing control per function (a valid TRUE order). Use one formula per row
  and the excel-probe skill (excel-graph, tags M7, lookup).

## Acceptance criteria
- [ ] probes/t4684-sort-false-order-and-range-key-errors.case.json is committed with excel-graph oracle values. It
      holds every form listed in Context.
- [ ] Every probe cell passes, or each cell that does not is listed in a new functions-lookup task (`campaign:
      semantics` with a `## Visible-parity stage` section) filed with `make new-task` that names the case and its
      cells. Those new known failures are recorded with `make eval-baseline`.
- [ ] arrays.md replaces both UNVERIFIED sentences with the observed rule and cites the probe cells.
- [ ] Any engine change has a unit test next to `fn_sort`/`fn_sortby` that pins the probed result.
- [ ] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- The by_col lifting rules (T-2319, T-2929) and the argument kinds of T-2874 (empty text and errors as SORT.by_col and
  SORTBY.sort_order themselves).
- Kana and width collation of the sort keys (T-3381).
