---
id: T-4736
title: "Aggregates over nested GROUPBY/PIVOTBY cells follow Excel's 2026-11 reads: SUM #N/A, COUNT skips, COUNTA one per cell"
milestone: M7
lane: engine
status: done
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
- t2625-nested-consumers P!A13, A17, A20, A25, A26;
- earlier-only, each confirmed by a committed refill (excel-graph, 2026-11-07, identical sheets): gen-m9/e85daedce5c2675e
  Calc!A35 (probes/gen-m9-e85daedce5c2675e-refill-1107) and probes/t2436/e85daedce5c2675e Calc!A31, A46
  (probes/t2436-e85daedce5c2675e-refill-1107). gen-m9/81bcdc0bec0c8e6b A121 and d8f44d556674c2fd A78 did not move.


Earlier-only candidates (decision 0329 item 3). These are cells with an October fill only that hold SUM or COUNT over a nested
result inside another formula. Supersede one only after you commit a refill copy of its case (identical sheets, as
probes/gen-m9-17c110a53924beb8-refill-1106 is; `excel-probe` skill) that shows the November value; otherwise keep it passing:
- gen-m9/e85daedce5c2675e Calc!A35 (`BESSELJ(COUNT(PIVOTBY(..),Data!B3:B5,Data!D9:D10),0)`, t2625 A26's COUNT);
- gen-m9/81bcdc0bec0c8e6b Calc!A121 (`IRR(SUM(..,PIVOTBY(..,LAMBDA(v,v),..),..))`) and gen-m9/d8f44d556674c2fd Calc!A78
  (`PROB(..,SUM(..,PIVOTBY(..,LAMBDA(y,y),..)),..)`).
Any other cell that the eval run shows moving is handled the same way, and it is listed here before the run.
- probes/t2436/e85daedce5c2675e Calc!A31 and A46: the same COUNT, bare and inside BESSELJ. Both pass today against their
  2026-10-24 fill (A31 142), so refill probes/t2436/e85daedce5c2675e first. T-2940's "ours 167" note for them is stale
  (review round 3 of T-4661).

## Acceptance criteria
- [x] Every cell under "fixes" passes; `make eval-show` shows each one.
- [x] Each earlier-only candidate that moves has a committed refill, and is superseded or kept passing as that refill shows.
- [x] The baseline diff adds F lines for exactly the cells under "supersedes", made by one
      `update-baseline --allow-regressions` run under decision 0329, and no other regression.
- [x] `NestedReading::Flatten` is removed, or kept only where a probe still shows flattening; arrays.md "Aggregates over
      nested GROUPBY and PIVOTBY cells" states the November reads as ours.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The element-wise lift reads (T-4737) and the lifted INDEX (T-4738).
- Aggregates whose nested reading did not change (`NotAvailable`, `Value`).

## Outcome
- `NestedReading::Flatten` is removed. SUM takes `NotAvailableReferenceValue` (`#N/A` for an array cell, the value for a
  reference-body cell), and COUNT and COUNTA take `NotAvailable`, as Excel's 2026-11 fills read nested GROUPBY/PIVOTBY
  cells (decision 0329). arrays.md states the November reads as ours. The unit tests in test_nested_groupby_cells.rs use
  them, under main's renamed helpers (`get_text_a1`, `set_a1`, T-4856).
- Refills (excel-graph, 2026-11-07) confirm the earlier-only cells: probes/gen-m9-e85daedce5c2675e-refill-1107 (400/400)
  and probes/t2436-e85daedce5c2675e-refill-1107 (12/12).
- Baseline, under decision 0329 item 3, with the owner's go-ahead (2026-11-07) for the `--allow-regressions` run:
  - after merging main (b54e1c1 and later), `update-baseline --suite probes --suite gen-m9 --allow-regressions` fixed 13
    cells and added F lines for 14 superseded cells. These are t2592 A6:A8, t2594 A26:A29, t2625 A13, A17, A20, A25, A26,
    and t2436/e85daedce5c2675e A31, A46. The 15th, gen-m9/e85daedce5c2675e Calc!A35, got its F line in eval/baseline/
    gen-m9.txt from this branch's first run. Against main, the PR adds F lines for exactly the 15 superseded cells;
  - the unrestricted `update-baseline --allow-regressions` run then raised exactly the five visible-parity lines by their
    superseded counts: gen-m9/e85daedce5c2675e +1, t2436/e85daedce5c2675e +2, t2592 +3, t2594 +4, t2625 1 -> 6. Every
    other suite reports 0 regressions;
  - the October sidecars are kept, not re-filled (decision 0329).
- History: the PR was blocked from 2026-11-07 05:xx to 23:4x UTC because the routine sessions' permission check refused
  the regression run. The owner allowed it in this session.
