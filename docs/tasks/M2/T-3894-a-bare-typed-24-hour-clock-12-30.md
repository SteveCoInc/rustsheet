---
id: T-3894
title: "A bare typed 24-hour clock (12:30) is a time with format h:mm, not text (T-3446 follow-up)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text typed into a cell that is a bare 24-hour clock with no date and no AM/PM (`12:30`) becomes the
time of day, as in Excel, instead of staying text.

## Context
probes/t3446-script-digits-forms B10: `12:30` typed into A10 is a number in Excel (ISNUMBER TRUE;
the oracle's ENTERED log gives value 0.520833… and format `h:mm`,
docs/research/t3446-script-digits-forms-2026-11-03.md). The engine leaves it text:
`parse_typed_number` (crates/rustsheet-engine/src/formatter/parse_formatted.rs) reads a bare clock
only with an `A`/`P`/`AM`/`PM` suffix (`parse_bare_letter_clock`), and `parse_formatted_number`
reads no clock at all. Coercion already reads it (`0+"12:30"`, through `parse_time_serial`).
docs/excel-semantics/dates.md "bare typed clock" lists the AM/PM shapes (T-2011, T-2053).

## Acceptance criteria
- [x] Probe (excel-probe skill) the bare 24-hour typed shapes beyond `12:30` before implementing any of
      them: `9:05`, `0:00`, `23:59`, `10:00:30`, `24:00`, `25:00`, `10:60`, `10:00.5`, `10:00:30.5`, `:30`,
      ` 10:00` (leading space), recording each value (ISNUMBER and the value via `=A1+0`) and the ENTERED
      format in a docs/research note.
- [x] `parse_typed_number` reads the probed number shapes with Excel's formats; probes/t3446-script-digits-forms
      B10 and the new probe's cells pass; dates.md records the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- Script digits in typed input (T-3891).

## Outcome
Probed 18 bare 24-hour shapes (probes/t3894-bare-24h-clock, excel-graph 2026-11-03; note
docs/research/t3894-bare-24h-clock-2026-11-03.md). `parse_bare_clock` in `parse_typed_number` reads them with
Excel's formats (`h:mm`, `h:mm:ss`, `[h]:mm:ss` from 24 hours, `mm:ss.0` for a fraction, General for an
out-of-range minute or second); `:30` and a leading-space clock stay text. Shapes beyond those probed (hour of 24
or more with seconds or an out-of-range minute) stay text. dates.md records the rule. Eval: the new probe passes
in full; probes/t3446-script-digits-forms B10 now passes (baseline updated).
