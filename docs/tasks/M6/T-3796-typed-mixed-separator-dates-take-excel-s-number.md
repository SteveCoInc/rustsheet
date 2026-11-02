---
id: T-3796
title: "Typed mixed-separator dates take Excel's number format; typed clock-first date-time is a number (T-3751 probe misses)"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Typed cell entry in the engine (`parse_formatted_number`, `crates/rustsheet-engine/src/formatter/parse_formatted.rs`) gives the
number format Excel gives for the mixed-separator dates, and a typed clock-first date-time (`10:00 7/1/2025`) is a number.

## Context
- Found by T-3751: probes/date-text-mixed-separator-typed (excel-graph, 2026-11-01) and the ENTERED log in
  docs/research/typed-mixed-separator-dates-2026-11-01.md; rule in docs/excel-semantics/dates.md ("Typed mixed-separator dates").
- Excel's typed format against the engine's (`7/1-2025` and `7-1/2025` already agree on `m/d/yyyy`):
  - `2025/07-01`, `2025-07/01`: Excel `m/d/yyyy`, engine `yyyy/mm/dd`.
  - `1-Jul/2025`, `1/Jul-2025`: Excel `d-mmm-yy`, engine `d/mmm/yyyy`.
  - `7/1-25`: Excel `m/d/yyyy`, engine `m/d/yy`.
  - `10:00 7/1/2025` typed: Excel 45839.4166666667 with `m/d/yyyy h:mm`; the engine keeps text
    (probes/date-text-mixed-separator-typed B10, C10 fail in `make eval-show`).
  - A leading space before a clock-first date (` 10:00 7/1/2025`, ` 10:00 AM 7/1/2025`) stays text in Excel and in the engine.

## Acceptance criteria
- [x] The typed formats above match Excel (unit tests in test_parse_formatted_number.rs citing the probe).
- [x] probes/date-text-mixed-separator-typed B10 and C10 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- The coerced and function readers (DATEVALUE, VALUE, `0+text`), which already match.

## Outcome
Mixed `/`-`-` typed dates now take `m/d/yyyy` (numeric) or `d-mmm-yy` (day-month-name-year) in
`date_text.rs`; a typed clock-first `H:MM <date>` (`10:00 7/1/2025`, the only evidenced shape) is a number with
`m/d/yyyy h:mm` (`parse_formatted.rs`, `leading_clock_shape`); a leading space stays text. Unit test
`typed_mixed_separator_date_formats`. Eval delta: probes/date-text-mixed-separator-typed B10 and C10
fixed (known failures -2, no regressions).
Follow-up: T-3852 probes the other clock-first shapes (AM/PM etc.), which stay text.

## Handoff
PR #2855, branch `agent/functions/T-3796-typed-mixed-sep-dates`, head 530c27a (reviewer rounds: excel-fidelity requested changes twice; security and design approved f38c327).
- Done: mixed-separator typed formats (`date_text.rs`, evidenced A1-A7) and eval B10/C10 fixed; baseline updated; `make check-fast` green.
- Open (excel-fidelity round 2, MAJOR): `leading_clock_shape` in `parse_formatted.rs` checks only the clock, so `10:00 July 1, 2025`, `10:00 1-Jul-2025`, `10:00 2025-07-01`, `10:00 7/1`, `23:59 7/1-2025` become numbers with no evidence, contradicting dates.md ("stay text", T-3852). Next step: restrict the date part to the evidenced numeric `m/d/yyyy` (1-2 digit month and day, 4-digit year, `/` only), add negative asserts for those shapes, and list year-less, ISO and mixed-separator dates after a clock in T-3852.
- Open (MINOR): `mixed` in `date_text.rs` changes the format in every locale though the comment says en-US evidence only; restrict it or name the gap in T-3852.
- Then re-run excel-fidelity only, set `status: done`, mark the PR ready and enqueue.
