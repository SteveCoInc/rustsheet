---
id: T-5102
title: "A WEEKNUM reader that RAND makes volatile is #NUM! in Excel's last recalc (weeknum-stale-volatile-reader Vol!A1)"
milestone: M6
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: foundational
depends_on: []
---

## Goal
`probes/weeknum-stale-volatile-reader` Vol!A1 matches Excel, or is recorded as an accepted
deviation with its reason.

## Context
- Found by T-5100 (decision 0357), which scores a random-reading cell when its engine value does
  not depend on the draws. Vol!A1 is `=WEEKNUM(2+0*RAND(),99)&WEEKNUM(3,99)&…&WEEKNUM(7,99)`
  above a Monday marker `=WEEKNUM(100,2)` in A2. The `0*RAND()` does not change the value. It is
  there to make the cell volatile, so Excel's last calculation of the oracle fill recalculates it
  on its own.
- Excel: `#NUM!` (excel-graph, 2026-10-19). Ours: `222222`, the value of a full recalc in which
  A1 reads the Monday state A2 left. dates.md "Recalcs and edits" and
  docs/research/t1484-weeknum-calc-order-2026-10-19.md ("Recalculated alone") explain it: a
  formula Excel recalculates alone sees an empty WEEKNUM state. SPEC-0017 leaves session history
  as a non-goal, since RustSheet recalculates the whole model on each `evaluate()`.
- So the gap is Excel's partial (volatile-only) recalc, not the draw. Baselined as new coverage
  by T-5100.

## Acceptance criteria
- [ ] Decide, with a decision record if it is a deviation: (a) model the oracle's last
      volatile-only recalc where it changes a value (a WEEKNUM reader that is volatile sees an
      empty state), or (b) list the cell in the probes suite's `deviations` file as session
      history outside SPEC-0017.
- [ ] Vol!A1 passes or is a listed deviation; `make check` green; reviewers approve.

## Out of scope
- Other session-history effects of T-1484 (edits, re-entered formulas).
