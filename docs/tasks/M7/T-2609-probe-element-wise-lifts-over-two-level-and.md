---
id: T-2609
title: "Probe element-wise lifts over two-level and reference-body GROUPBY cells (SPEC-0018 rules 4, 7, 8)"
milestone: M7
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
arrays.md and SPEC-0018 state, from oracle values, what an element-wise lift reads from a two-level nested GROUPBY cell and from a
reference-body cell.

## Context
SPEC-0018 (revised by T-2602) assumes that `ArrayNode::scalar()` reads a two-level nested cell's top-left recursively and that a
reference-body cell (`LAMBDA(x,D!D2)`) is `#VALUE!` to every read but INDEX (rule 7, probes/t2592-nested-groupby-cells A21).
Neither is probed. Probe with the t2592 data sheet D (`GROUPBY(D!A1:A5,D!C1:D5,..)`):
- `SUM(--ISERROR(GROUPBY(D!A1:A5,D!C1:D5,LAMBDA(x,D!D2))))`, `SUM(--ISNUMBER(..))` and `ROWS(..)`;
- the same over the A23 two-level formula, a LAMBDA returning a GROUPBY whose inner top-left is an error;
- `REGEXEXTRACT("a1",GROUPBY(..,LAMBDA(x,D!D2)))`;
- SUM, COUNT and COUNTA over the reference-body result;
- INDEX of a reference-body cell whose referenced cell is blank or an error (T-2593 holds the blank or the error, unprobed);
- an operator over the whole result read element by element (`SUM(GROUPBY(..,LAMBDA(v,v))+0)`, `ISERROR(GROUPBY(..)+0)`) and a
  lifted INDEX element of a nested cell (`TYPE(INDEX(GROUPBY(..,LAMBDA(v,v)),{2;3},2))`): T-2593 reads both as `#VALUE!`.

Partial evidence already on file (T-2623): over a `LAMBDA(z,z)` result, `MEDIAN({1;2;"";4},GROUPBY(..)-x,..)` is #N/A in Excel
and `TEXTJOIN(";",FALSE,IFERROR((GROUPBY(..)-x)&"",..))` fails (probes/t2475-stat-subexpressions P!A62, A63); ours reads each nested
cell as `#VALUE!` and gives #VALUE!. The fix is T-2686.
Record the answers in arrays.md (the T-2592 paragraph) and update SPEC-0018's "One reader" and rule 7. If the evidence changes
what T-2593 or T-2594 must build, update those task bodies too.

## Acceptance criteria
- [x] A probe case with oracle values for the formulas above, cited in arrays.md and SPEC-0018.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-2593, T-2594).

## Outcome
Landed with T-2686 (one PR). Three probe cases, predictions in column B committed before each fill (excel-graph, 2026-11-06):
probes/t2609-nested-operators-and-lifts (49 cells: operators over the whole result, two-level, reference-body and lifted-INDEX
reads), -one-cell (30 cells: the same operators over 1x1 nested cells) and -d04 (17 cells: per-cell TYPE and shape through MAP of
the gen-m9/17c110a53924beb8 GROUPBY, and t2475 P!A62, A63 again). Findings (arrays.md "Operators over nested cells, and Excel's
change of 2026-11", SPEC-0018 rules 4 and 7):
- An operator over the whole result makes every nested cell `#VALUE!` (any size, kind, depth, operator): ours already does.
- Excel changed some nested-cell reads between 2026-10-28 and 2026-11-06. Byte-identical copies of the t2592, t2594, t2625, t2360
  and t2311 probes, refilled today (`<case>-refill-1106`, two identical fills for t2592 and t2594), differ from their October
  fills only where an element-wise lift reads a nested cell (now an error, not the top-left) and where SUM (now `#N/A`), COUNT
  (now skips) and COUNTA (one per cell) read the result. Uncommitted refills of t2475-stat-subexpressions (same input sha256) and
  of gen-m9/17c110a53924beb8 change only P!A62:A64 and Calc!A300, to ours.
- A two-level cell is an error to a lift (not its inner top-left, the SPEC-0018 assumption); a reference-body cell is `#VALUE!`
  to a lift and an operator, its value to SUM (60), 0 to COUNT; a lifted INDEX of nested cells gives each one's top-left.
T-2593 and T-2594 are done; the revision of their rules, and the cells it owes, are T-4661 (filed here). Eval: 8 new probe cases,
28 new-coverage failures (all owned by T-4661), no regression.
