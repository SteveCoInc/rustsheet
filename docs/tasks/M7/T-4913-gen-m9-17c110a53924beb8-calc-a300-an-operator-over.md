---
id: T-4913
title: "gen-m9/17c110a53924beb8 Calc!A300: an operator over a raw GROUPBY takes its 2026-11 #VALUE! (decision 0329)"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
gen-m9/17c110a53924beb8 Calc!A300 leaves eval/baseline/gen-m9.txt: under decision 0329 the 2026-11-06 refill, which
gives the `#VALUE!` ours returns, is the target, and the October `#N/A` fill no longer counts against us.

Owned cell: gen-m9/17c110a53924beb8.case.json Calc!A300

## Context
Filed by the 2026-11-07 steward PR (#3872), which first named this cell in T-4737. That was wrong: T-4737 merged as
#3863 without it, and its Out of scope excludes "Operators over the whole result". `make unowned-off` found the cell
with no open owner once T-2686 was done.
- October fill (gen-m9/17c110a53924beb8.oracle.json): Calc!A300 is `#N/A`, an operator over a raw GROUPBY.
- 2026-11-06 refill (probes/gen-m9-17c110a53924beb8-refill-1106.oracle.json): Calc!A300 is `#VALUE!`, which ours
  returns today (T-2686's Outcome).
- eval/baseline/gen-m9.txt still has `F gen-m9/17c110a53924beb8.case.json Calc!A300`.
Decision 0329 says how a cell is superseded (item 3); T-4736 and T-4737 are worked examples of the same move for
nested cells read by lifts.

## Acceptance criteria
- [ ] The cell is superseded per decision 0329 item 3 and its F line leaves eval/baseline/gen-m9.txt through
      `make eval-baseline` (never by hand).
- [ ] docs/excel-semantics names the 2026-11 `#VALUE!` for an operator over a raw GROUPBY, citing the refill.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change: ours already returns the November value.
