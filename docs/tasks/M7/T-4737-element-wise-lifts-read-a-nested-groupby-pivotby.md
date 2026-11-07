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
- t2475-stat-subexpressions P!A64 (October `64|13x3|?`; ours passes it today);
- gen-m9/17c110a53924beb8 Calc!A300 (October `#N/A` for an operator over a raw GROUPBY; its 2026-11-06 refill,
  probes/gen-m9-17c110a53924beb8-refill-1106, gives the `#VALUE!` ours returns, T-2686's Outcome; named here by the
  steward 2026-11-07 because `make unowned-off` found no open owner).

Earlier-only candidates (decision 0329 item 3). These are cells with an October fill only that read a nested cell through a
lift. Supersede one only after you commit a refill copy of its case that shows the November value; otherwise keep it
passing:
- gen-m7/21b4eef18424ce94, whose REGEXEXTRACT reads its return_mode through a nested cell. A simulation of this change
  (T-4661 review round 3: `scalar` returning `#VALUE!`) leaves A1504 alone but moves 26 of its spill cells:
  Calc!B1505, B1506, B1510, B1512, B1513, C1505, C1507:C1511, C1514, D1505, D1507, D1509:D1511, D1513, E1505:E1507 and
  E1509:E1512.
- The same simulation leaves gen-m7/b24ce74f08d3f91a A1826 and probes/rich-errors-regex-group A23, K23, L23 unmoved.
- The lift driver reads every lifted function's elements through `scalar()` (functions/lift/driver.rs,
  `LiftedArg::element`), so this change reaches every element-wise lift. The November evidence covers ISERROR, ISNA,
  ISTEXT, ISNUMBER, ERROR.TYPE and REGEXEXTRACT only.
Any other cell that the eval run shows moving is handled the same way, and it is listed here before the run.

## Acceptance criteria
- [ ] Every cell under "fixes" passes; `make eval-show` shows each one.
- [ ] The baseline diff adds F lines for exactly the cells under "supersedes", made by one
      `update-baseline --allow-regressions` run under decision 0329, and no other regression.
- [ ] The error a lift reads is evidenced by a probe and cited where `scalar` (or its replacement) is defined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The aggregates (T-4736) and the lifted INDEX (T-4738).
- Operators over the whole result: they already give `#VALUE!` per nested cell.
