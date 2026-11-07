---
id: T-4661
title: "Revise SPEC-0018 for Excel's 2026-11 nested-cell reads (lifts read an error, SUM is #N/A, COUNT skips)"
milestone: M7
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SPEC-0018 states the reads of a nested GROUPBY/PIVOTBY cell that current Excel shows (2026-11-06 fills), a decision record says
which fill the eval targets for the cells whose Excel value changed, and engine tasks implement the revision.

## Context
Found by T-2609 (docs/excel-semantics/arrays.md "Operators over nested cells, and Excel's change of 2026-11"). Byte-identical
copies of the October nested-cell probes, refilled from excel-graph on 2026-11-06 (twice for t2592 and t2594, identical), agree with
their October fills except in these reads, where ours follows October (SPEC-0018 rule 4, `ArrayNode::scalar` and the walker's
`NestedReading` in functions/util/aggregate_args.rs):
- SUM over a result holding nested cells is `#N/A` (was a flattened sum), COUNT skips the nested cells (was a flattened count) and
  COUNTA counts each as one value;
- an element-wise lift reads a nested cell as an error, not as its top-left (`SUM(--ISERROR(GROUPBY(..)))` counts every nested cell);
- an operator over the whole result makes each nested cell `#VALUE!`: ours already does (T-2609 probes, all cells pass), but the
  October fills of the gen-m9 and t2475 cells below show the earlier behavior.
New cases owned here (every off cell):
- probes/t2592-nested-groupby-cells-refill-1106
- probes/t2594-nested-aggregates-refill-1106
- probes/t2625-nested-consumers-refill-1106
- probes/t2360-array-lambda-holidays-refill-1106
- probes/t2311-pivotby-value-columns-error-refill-1106
- probes/t2609-nested-operators-and-lifts P!A44:A46 (two-level cells read as an error by a lift), P!A50 (SUM over a reference-body
  result reads the values, 60), P!A56:A59 (a lifted INDEX of nested cells gives each one's top-left)
- probes/t2609-nested-operators-one-cell P!A27 (a lift over a 1x1 cell holding a nested array)
- probes/t2475-stat-subexpressions-refill-1106 P!A64 (a lifted `ERROR.TYPE`/`&""` listing: 2026-11-06 a full listing, ours `?` as in
  October)
October cells that today's Excel answers as ours does (refills of byte-identical copies on 2026-11-06, committed as
probes/t2475-stat-subexpressions-refill-1106 and probes/gen-m9-17c110a53924beb8-refill-1106), handed from T-2686:
- probes/t2475-stat-subexpressions P!A62, P!A63 (2026-10-28 `#N/A` and `?`; 2026-11-06 `#VALUE!` and our 13x3 listing)
- gen-m9/17c110a53924beb8 Calc!A300 (2026-10-23 `#N/A`; 2026-11-06 `#VALUE!`, ours)
Every fill on both dates is excel-graph (Excel for the web); the decision record weighs whether a desktop or second-backend fill must
confirm the change before a November value becomes the target (oracle-calibration.md's reproducibility floor, decision 0039, T-3610).
The old values stay committed (T-3610's draft rule: re-filling until a run matches us is ruled out; decision 0284 precedent). Moving
the engine to the November reads turns the October cells of the original probes (t2592 A6:A8, t2594 A26:A29, t2625 A13, A17, A20,
A25, A26, A32, t2360 P!C1, C9, C17, C25, C33, t2311 P!C12) into known failures, so the switch needs this task's decision record
(an Excel behavior change, CLAUDE.md directive 1 "current Microsoft 365 Excel"; relate it to T-3610).

## Acceptance criteria
- [x] A decision record says which fill is the target for cells whose Excel value changed between dated fills, and how the
      superseded cells are scored.
- [x] SPEC-0018 rules 4 and 7 and "One reader" state the November reads with the cases above as evidence; engine tasks are filed
      for the change, each owning its cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- The engine change itself (the filed tasks).

## Outcome
Done 2026-11-07.
- **Decision 0329:** a later fill is a behavior change when a byte-identical copy reproduces it, either on two
  agreeing fills or under one stated rule across at least two probes, and the copy's other cells agree. The later fill
  is then the target, with no desktop confirmation needed (decision 0039). The earlier sidecar stays committed. Its
  changed cells are superseded: they become F lines through one scoped `--allow-regressions` run in the task that moves
  the engine, as decision 0284 did. Variation with no rule behind it stays T-3610's.
- **SPEC-0018:** rule 4 now states the 2026-11-06 reads, and the October reads are listed as superseded. Rule 7 notes
  which reads come from November fills, "One reader" says how `scalar` changes, and the plan lists the three engine
  tasks. arrays.md's "Excel's change of 2026-11" entry points to the decision and the tasks.
- **Engine tasks filed:**
  - T-4736: aggregates (SUM `#N/A`, COUNT skips, COUNTA one per cell), the t2592, t2594 and t2625 cells, and t2609
    A50;
  - T-4737: lifts read an error, the t2360, t2311 and t2625 A32 cells, t2609 A44:A46, -one-cell A27 and the t2475
    refill's A64;
  - T-4738: the lifted INDEX top-left, t2609 A56:A59.
  Each task lists the cells it fixes and the cells it supersedes.
- **Eval delta:** none (docs only).
