---
id: T-4283
title: "Pin XIRR's whole-day date bound (2958465.5 is a date, -0.5 is #NUM!) and point financial.md at flows.rs"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3312]
---

## Goal
A test pins XIRR's pair-order date check, which compares the date truncated to a whole day, so a later change that
compares the raw date fails the gate. financial.md points at the file the XIRR/XNPV readers now live in.

## Context
These are the MINORs from T-3312's (#3247) round-2 reviews, raised after approval:
- `crates/rustsheet-engine/src/functions/financial/flows.rs` (`xirr_flows`) checks
  `date.floor() < 0.0 || date.floor() > MAXIMUM_DATE_SERIAL_NUMBER`, as `xirr_dates` does (it truncates first). So
  2958465.5 is a valid date and -0.5 is `#NUM!`. Round 1 caught the raw-date version, which made 2958465.5 `#NUM!`.
  No unit test or eval cell has a fractional date at or near 2958465 or in (-1, 0). Only XNPV's "fractional dates
  truncate" has evidence (docs/excel-semantics/financial.md, the T-2599 entry).
- docs/excel-semantics/financial.md, the two T-3312 entries: their "Implemented in" lines still name
  `financial/mod.rs` for `xirr_flows`, `xnpv_flows` and `xnpv_dates`. Those now live in `financial/flows.rs`.
- The `xnpv_numbers` doc comment in `financial/mod.rs` has one over-long line (needs re-wrapping).
- T-3312's first acceptance criterion says "no new known-failure lines", but the PR added
  `probes/t3312-xirr-xnpv-inner-calls Calc!A30` (owned by T-4251). The Outcome discloses it. Reword the Outcome's
  note so the two do not contradict each other.

## Acceptance criteria
- [x] A unit test: XIRR whose last date is 2958465.5 gives a number, and one with a date of -0.5 gives `#NUM!`.
- [x] A probe cell for each of those two inputs, oracle-filled (excel-probe skill). If Excel differs, record the
      quirk in financial.md and match it.
- [x] financial.md's "Implemented in" lines name `flows.rs`, the doc comment is re-wrapped, and T-3312's Outcome
      note no longer contradicts its criterion.
- [x] `make check` green; reviewers approve.

## Out of scope
- `XIRR({-100,200,-300},{1,2,3})` (T-4251).

## Outcome
Probed `probes/t4283-xirr-whole-day-date-bound` (10 cells, excel-graph): Excel checks XIRR's date bound on the truncated date, so 2958465.5 and 0.5 are dates and 2958466 and -0.5 are `#NUM!`; the engine already matched. Added `fn_xirr_date_bound_is_checked_on_the_whole_day`, a financial.md entry, and re-pointed the two T-3312 "Implemented in" lines at `flows.rs`. Re-wrapped the `xnpv_numbers` doc comment and reworded T-3312's criterion so it no longer contradicts its Outcome (A30, owned by T-4251). Eval delta: +10 passing probe cells, no regressions.
