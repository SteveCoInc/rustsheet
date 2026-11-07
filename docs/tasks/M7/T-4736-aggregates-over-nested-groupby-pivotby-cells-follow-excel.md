---
id: T-4736
title: "Aggregates over nested GROUPBY/PIVOTBY cells follow Excel's 2026-11 reads: SUM #N/A, COUNT skips, COUNTA one per cell"
milestone: M7
lane: engine
status: in-progress
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
- [ ] Every cell under "fixes" passes; `make eval-show` shows each one.
- [ ] Each earlier-only candidate that moves has a committed refill, and is superseded or kept passing as that refill shows.
- [ ] The baseline diff adds F lines for exactly the cells under "supersedes", made by one
      `update-baseline --allow-regressions` run under decision 0329, and no other regression.
- [ ] `NestedReading::Flatten` is removed, or kept only where a probe still shows flattening; arrays.md "Aggregates over
      nested GROUPBY and PIVOTBY cells" states the November reads as ours.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The element-wise lift reads (T-4737) and the lifted INDEX (T-4738).
- Aggregates whose nested reading did not change (`NotAvailable`, `Value`).

## Landing notes
- Branch agent/engine/T-4736-nested-aggregates-november, no PR yet (opened by the 2026-11-07 opus run at its landing window).
- Done: `NestedReading::Flatten` removed. SUM takes `NotAvailableReferenceValue` (`#N/A` for an array cell, the value for a
  reference-body cell); COUNT and COUNTA take `NotAvailable`. The two refills are committed with their oracle sidecars
  (excel-graph) and pass in full. `rustsheet-eval update-baseline --suite probes --suite gen-m9 --allow-regressions
  --new-cases <the two refill case files>` ran once: eval/baseline/probes.txt and gen-m9.txt drop the 13 fixed F lines
  (12 listed + t2609 A50), add F lines for exactly the 15 cells under "supersedes", and add C lines for the two refills.
  `make eval-check` then shows 0 known-failure regressions.
- Open: `make eval-check` still fails the visible-parity lock: 5 cases rise by 1, 2, 3, 4 and 5, which is exactly their
  superseded-cell counts (gen-m9/e85d 1, t2436/e85d 2, t2592 3, t2594 4, t2625 5). Decision 0329 item 3 lets the same
  regression run raise these lines. The `--suite`-restricted run did not touch eval/baseline/visible-parity.txt. A second,
  unrestricted `update-baseline --allow-regressions` run was refused by this session's sandbox permission check, so do it
  in a session where it is allowed, and check that its visible-parity diff is only those 5 lines.
- Unit tests: test_nested_groupby_cells.rs moved to the November reads (`aggregates_read_nested_cells_as_errors`; the bound
  test keeps REDUCE's `#ERROR!` and now expects SUM `#N/A`). `cargo test -p rustsheet_engine`: 5687 passed, 0 failed;
  clippy and fmt clean.
- Then: raise the visible-parity lines as above, `make check-fast`, the review panel (design, security, excel-fidelity),
  set `status: done` with an Outcome, and open the PR. Do not re-fill or edit the October sidecars (decision 0329).
