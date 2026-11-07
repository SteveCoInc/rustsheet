---
id: T-4736
title: "Aggregates over nested GROUPBY/PIVOTBY cells follow Excel's 2026-11 reads: SUM #N/A, COUNT skips, COUNTA one per cell"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUM, COUNT and COUNTA over a GROUPBY or PIVOTBY result that holds nested cells give the values Excel gave on
2026-11-06: SUM is `#N/A`, COUNT skips the nested cells, and COUNTA counts each nested cell as one value. SUM over a
reference-body result reads each cell's value.

## Context
Decision 0329 makes the 2026-11-06 fills the target. The rule is in SPEC-0018 rule 4 (revised by T-4661) and in
docs/excel-semantics/arrays.md "Operators over nested cells, and Excel's change of 2026-11". Today the walker flattens
nested cells for SUM, COUNT and COUNTA (`NestedReading::Flatten`, functions/util/aggregate_args.rs; count.rs and the SUM
policy), which is the October behavior.

Cells this task fixes (probes):
- t2592-nested-groupby-cells-refill-1106 P!A6:A8: SUM `#N/A`, COUNT 0, COUNTA 9;
- t2594-nested-aggregates-refill-1106 P!A26:A29: `SUM(1,GROUPBY(..),2)` `#N/A`, SUM `#N/A`, COUNT 0, and COUNTA over
  PIVOTBY 28;
- t2625-nested-consumers-refill-1106 P!A13 (COUNT over PIVOTBY 90), A17 (SUM `#N/A`), A20 and A25 (COUNT over an
  INDEXed column or row, 1), A26 (COUNT with two more ranges, 92);
- t2609-nested-operators-and-lifts P!A50: SUM over a reference-body result is 60 (rule 7).

Cells this task supersedes (decision 0329 item 3; they become F lines against their October values):
- t2592-nested-groupby-cells P!A6:A8;
- t2594-nested-aggregates P!A26:A29;
- t2625-nested-consumers P!A13, A17, A20, A25, A26.

## Acceptance criteria
- [ ] Every cell under "fixes" passes; `make eval-show` shows each one.
- [ ] The baseline diff adds F lines for exactly the cells under "supersedes", made by one
      `update-baseline --allow-regressions` run under decision 0329, and no other regression.
- [ ] `NestedReading::Flatten` is removed, or kept only where a probe still shows flattening; arrays.md "Aggregates over
      nested GROUPBY and PIVOTBY cells" states the November reads as ours.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The element-wise lift reads (T-4737) and the lifted INDEX (T-4738).
- Aggregates whose nested reading did not change (`NotAvailable`, `Value`).
