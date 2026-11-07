---
id: T-4737
title: "Element-wise lifts read a nested GROUPBY/PIVOTBY cell as an error (Excel 2026-11), not its top-left"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An element-wise lift over a GROUPBY or PIVOTBY result reads each nested cell as an error, as Excel did on
2026-11-06, and not as its array's top-left. The error is `#VALUE!`.

## Context
Decision 0329 makes the 2026-11-06 fills the target. The rule is in SPEC-0018 rule 4 and "One reader" (revised by
T-4661) and in docs/excel-semantics/arrays.md "Operators over nested cells, and Excel's change of 2026-11". Today
`ArrayNode::scalar` (nested_array.rs) returns a nested cell's top-left, and the element-wise lifts (ISERROR,
REGEXEXTRACT's return_mode) read through it. That is the October behavior.

The error is `#VALUE!` for one-level cells: the lifted fallback `"#"&ERROR.TYPE(G)` of `IFERROR(G&"",..)` gives "#3"
at every nested cell, 1x1 and multi-cell (t2475-stat-subexpressions-refill-1106 P!A64 over a raw GROUPBY,
t2625-nested-consumers-refill-1106 A32 over `INDEX(PIVOTBY(..),4,0)`). A reference-body cell reads as `#VALUE!` too
(t2609-nested-operators-and-lifts A41, A42, A49). For a two-level cell only "not `#N/A`" is shown: ISNA is 0 and
ISERROR TRUE where the inner top-left is `#N/A` (A44:A47). Probe its ERROR.TYPE with the `excel-probe` skill before
assuming `#VALUE!` there.

Cells this task fixes (probes):
- t2360-array-lambda-holidays-refill-1106 P!C1, C9, C17, C25, C33 (`SUM(--ISERROR(PIVOTBY(..)))` counts every nested
  cell);
- t2311-pivotby-value-columns-error-refill-1106 P!C12;
- t2625-nested-consumers-refill-1106 P!A32 (the lifted ERROR.TYPE fallback over `INDEX(PIVOTBY(..),4,0)`: "#3" at each
  nested cell);
- t2609-nested-operators-and-lifts P!A44:A46 (two-level cells read as an error by a lift);
- t2609-nested-operators-one-cell P!A27 (a lift over a 1x1 cell holding a nested array);
- t2475-stat-subexpressions-refill-1106 P!A64 (the same pattern over a raw GROUPBY: a full listing with "#3" cells).

Cells this task supersedes (decision 0329 item 3):
- t2360-array-lambda-holidays P!C1, C9, C17, C25, C33;
- t2311-pivotby-value-columns-error P!C12;
- t2625-nested-consumers P!A32;
- t2475-stat-subexpressions P!A64 (October `64|13x3|?`; ours passes it today).
Check gen-m7/21b4eef18424ce94 Calc!A1504 too: REGEXEXTRACT reads its return_mode through a nested cell. It has no
November fill. If the change moves it, file a refill probe before you supersede it.

## Acceptance criteria
- [ ] Every cell under "fixes" passes; `make eval-show` shows each one.
- [ ] The baseline diff adds F lines for exactly the cells under "supersedes", made by one
      `update-baseline --allow-regressions` run under decision 0329, and no other regression.
- [ ] The error a lift reads is evidenced by a probe and cited where `scalar` (or its replacement) is defined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The aggregates (T-4736) and the lifted INDEX (T-4738).
- Operators over the whole result: they already give `#VALUE!` per nested cell.
