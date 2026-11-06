---
id: T-4631
title: "Re-decide ACCRINT at the calendar's start after Excel for the web's 2026-11-06 change"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ACCRINT at the calendar's start follows Excel once fills agree over time: either the committed `#NUM!` rule stays, or
the engine, the Gate A definition and the probes' sidecars move to the 2026-11-06 answers (0 when the walk back to the
issue reaches a period start before 1900-01-01; no basis-1 regular-period cut; no 1904 cut except for a settlement's
own period).

## Context
- financial.md "ACCRINT at the calendar's start", bullet "Re-fill of 2026-11-06 (T-4566)": two fresh fills that day
  agreed with each other and differed from the committed 2026-10-20 to 10-27 sidecars in 140 cells across
  probes/accrint-zero-results, fin-accrint-1900-calendar, fin-coupon-walk-bounds-b, -c, accrint-calendar-start-1900,
  accrint-calendar-start-1904 and accrint-eom-1900. Excel for the web gave `#NUM!` on 2026-10-20 to 10-27 and 0 on
  11-06 for about 140 cells (fin-coupon-walk-bounds-b A9, the only cell filled on 10-17, gave 0 then), so T-4566 followed neither and kept the sidecars and `compute_accrint` as they were.
- The 11-06 answers also contradict tools/src/rustsheet_tools/accuracy/defs/accrint.py's `#NUM!` domain for calc_method
  FALSE or a settlement before first_interest when only the walk back (not the settlement's period) starts before
  1900-01-01, and its basis-1 regular-period cut.
- Known failures carried for this: probes/accrint-zero-results A1:A9, A12, A14, A16:A19 (eval baseline).

## Acceptance criteria
- [ ] Re-fill the seven probes above with `rustsheet-oracle fill --force` and an emptied cache, at least twice, no
      sooner than 2026-11-20; record the dates and values in financial.md.
- [ ] If those fills agree with 2026-11-06, commit the new sidecars and make `compute_accrint` and defs/accrint.py
      follow them; if they agree with the committed sidecars, refill accrint-zero-results to match; otherwise record
      the instability and leave the cells as known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ACCRINT's unprobed points (T-4522).
