---
id: T-3903
title: "Probe bare typed 24-hour clocks with an hour of 24 or more and seconds or an out-of-range minute (T-3894 follow-up)"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Bare typed 24-hour clocks beyond T-3894's probed shapes (`25:00:30`, `25:60`, `100:00:00`, `24:00:00`, and the out-of-range minute shapes `10:60:00`, `10:61`, `10:99`, `0:00:75`, which T-3894 types as General by extrapolation from `10:60` and `10:00:60`) are typed as Excel does instead of staying text.

## Context
`parse_bare_clock` (crates/rustsheet-engine/src/formatter/parse_formatted.rs) leaves these text because T-3894's probe did not cover them; dates.md "bare typed clock" records the probed shapes.

## Acceptance criteria
- [x] Probe the shapes (excel-probe skill), record value and ENTERED format in a docs/research note, and extend `parse_bare_clock` to match.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 21 shapes (probes/t3903-bare-24h-clock-extra, excel-graph). `parse_bare_clock` now types an hour >= 24 with seconds as `[h]:mm:ss`; hour >= 24 with an out-of-range minute/second stays text; hour < 24 out-of-range was already General. Unit test and dates.md updated.
