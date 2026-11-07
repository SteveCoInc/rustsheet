---
id: T-3805
title: "exit_owners: leave the desktop-owned cells the Phase 1 rate leaves out (decision 0143) out of M9's rate: criterion"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`make exit-owners MILESTONE=M9` lists under `rate:<FUNCTION>` only the visibly-off cells that the Phase 1 rate counts.
It no longer lists the cells the rate leaves out as desktop-owned (decision 0143), so a triager does not read them
as gaps with no owner.

## Context
- Gap triage 2026-11-02 (`make eval` on origin/main 56917e82c). `make exit-owners MILESTONE=M9 STRICT=1` reports
  272 unowned FORECAST.ETS-family cells: `rate:FORECAST.ETS.STAT` 153, `rate:FORECAST.ETS` 49,
  `rate:FORECAST.ETS.CONFINT` 47 and `rate:FORECAST.ETS.SEASONALITY` 23. Those are exactly the `desktop_owned` counts
  of the four functions in report.json's `v15_checkpoint.functions`. These are gen-m9 cells (Excel `#N/A`, ours
  `#N/IMPL!`) that only T-2425 (M14) owns. `make eval` writes them to eval/runs/desktop-owned.tsv, and
  `rustsheet-eval run --desktop-owned` leaves them out of the rate. They block no exit, but the list says they have
  no owner.
- Cause: `counted_in_rate` (scripts/exit_owners.py) leaves out only the `r3_pass` and `phase1b` classes of the
  report's `r1c_left_out` (`RATE_LEFT_OUT`). Its docstring says the Rust rate "handles those through its own lists".
  But `r1c_left_out` marks `m14_owned` only on probe-row cells (`r1c.rs` `classify`, decision 0188 R7). So gen-m9
  desktop-owned cells reach the `rate:` list.
- Fix: read the desktop-owned list the rate itself reads, from eval/runs/desktop-owned.tsv, or better from a list
  that `rustsheet-eval` writes into report.json beside the count. Leave a `rate:` cell out when it is on that list.
  One definition: reuse `scripts/desktop_owned.py`'s reader or the report field, and never recompute ownership. The
  other criteria (gen suite, ironcalc-calc rows, probe rows) do not change.
- Tests: scripts/test_exit_owners.py (and scripts/test_desktop_owned.py if the reader moves).

## Acceptance criteria
- [x] A unit test in scripts/test_exit_owners.py: a below-rate function's visibly-off cell on the desktop-owned list
      is not in `exit_cells(report, "M9")`, and a cell not on the list still is.
- [x] After `make eval` on a tree with this change, `make exit-owners MILESTONE=M9` lists no `rate:FORECAST.ETS*`
      cell whose case!cell line is in eval/runs/desktop-owned.tsv. The PR shows the count before and after.
- [x] `make exit-owners` for M2, M5, M6, M7, M8 and M10 gives the same output as before.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing which cells are desktop-owned (`desktop_owned.py`'s rule) or the Rust rate.

## Outcome
`exit_owners.exit_cells` now leaves a `rate:` cell out when its `case!Sheet!A1` is on
eval/runs/desktop-owned.tsv, the list `make eval` writes and `rustsheet-eval run --desktop-owned`
reads (`read_desktop_owned`; no ownership is recomputed). Other criteria are untouched. Unit test
`test_desktop_owned_cells_leave_the_rate` added. Decision 0251 records the rule (the scripts are
protected paths). `make eval` on origin/main 2e781deba: `make exit-owners MILESTONE=M9` went from 90
unowned `rate:FORECAST.ETS*` cells to 0 (STRICT=1 likewise lists none); M2, M5, M6, M7, M8 and M10
output is byte-identical before and after. Eval delta: none (scripts only).

## Handoff (resumed)
Implemented and both reviewers (design, security) approved head e1dce6b9 (PR #2853). CI `hygiene` fails
`check_governance.py`: `scripts/exit_owners.py` and `scripts/test_exit_owners.py` are protected paths
and the PR carries no decision record. A sonnet run may not add one. Next: an opus run adds a decision
record (`make new-decision TITLE="..."`, ruling that M9's `rate:` list leaves out the desktop-owned
list per decision 0143), re-runs the reviewers on the new head, confirms the `make eval` before/after
count (272 `rate:FORECAST.ETS*` cells expected to drop to 0), then enqueues. The code needs no change.
