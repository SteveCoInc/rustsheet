---
id: T-3983
title: "TIME wraps an argument below -32768 by 65536 (Excel drift since 2026-10-21)"
milestone: M6
lane: functions-date
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-3678]
---

## Goal
TIME reads a whole argument below -32768 the way Excel does now: it wraps by 65536 into -32768..32767, so
`TIME(600,-32769,0)` is `TIME(600,32767,0)`. An argument above 32767 stays `#NUM!`.

## Context
- dates.md, "TIME: an argument past 32767 is `#NUM!`" (the 2026-11-03 update): Excel for the web changed after
  2026-10-21. probes/t3678-time-argument-far-negative (excel-graph, 2026-11-03, T-3678) has the cells:
  - A1:F14 cover the minute, second and hour from -32769 down to -131072.
  - Column G repeats the 2026-10-21 cells that were `#NUM!` then and are numbers now: time-argument-bound A22:A24,
    time-argument-window A73, A75 and A76, and time-argument-negative A65 and A66.
- The near-integer read comes first (`near_integer_whole`), then the wrap: `TIME(600,-32768.5,0)` floors to
  -32769 and wraps to 32767. -32768 itself is in range (`TIME(-32768,0,0)` is `#NUM!` only because its total is
  negative). A wrapped value can still make the total negative: `TIME(-65537,0,0)` wraps to -1 and is `#NUM!`.
- Code: `time_argument` in `crates/rustsheet-engine/src/functions/date_and_time.rs`.
- Gate A: `accuracy/TIME/s1!Calc!A149` (`TIME(14120,-33313.544720951,-1389.80591442708)`, Excel 0.6936342592592837,
  the engine `#NUM!`) is listed in `eval/accuracy/known-failures.tsv` under this task. Remove that line once it
  passes.
- The older probe sidecars (time-argument-bound A22:A24, time-argument-window A73, A75 and A76,
  time-argument-negative A65 and A66) still hold the 2026-10-21 `#NUM!`. Refill those three cases from excel-graph
  (`rustsheet-oracle fill --force`) in the same PR, so the engine change does not regress them.
- Unprobed: arguments below -131072. Does the wrap still hold there, or does a conversion to a 32-bit integer cut
  in? Probe a few (-1E6, -2^31, -1E15) before the change, so the engine claims nothing that is unevidenced.

## Acceptance criteria
- [ ] `time_argument` wraps a whole value below -32768 by 65536. Unit tests cover the cells above.
- [ ] The three older probe cases are refilled. The t3678 probe and the TIME/s1 table pass. The known-failures
      line is removed.
- [ ] dates.md's entry says "Ours: matches".
- [ ] `make check` green; reviewers approve.

## Out of scope
- DATE's argument bounds: they are a separate rule (dates.md "DATE month and day bounds"). Probe them in their own
  task if they are suspected of the same drift.

## Outcome
`time_argument` wraps a whole value below -32768 by 65536 and reads one below -2^31 as 0. Probed
below -131072 (probes/t3983-time-argument-below-131072, excel-graph 2026-11-04): the wrap holds to -2^31;
beyond it Excel gives 0, not the wrap. Refilled time-argument-bound/-window/-negative. Removed the
Gate A known-failure TIME/s1 A149. dates.md says "Ours: matches". Unit tests added in test_fn_time.rs.
Also refilled four gen-m6/gen-m9 cases whose 2026-10 sidecars held the pre-wrap `#NUM!` (1a404a0379a1372c A8, 44194b2fefa30de2 A251, 6684de07ac74bcb8 A343, 09b47629f54b9817 A70). Eval delta: probes known failures 4433 -> 4352 (81 cells fixed), no regressions elsewhere.

Adopted 2026-11-04 by an opus run (orphan, step 0.3): `make catch-up` cleared the probes baseline conflict; added the
missing math sidecar of t3983-time-argument-below-131072 (all 24 cells are generator skips, "no single value");
dates.md's two stale sentences are reworded. Security's MINOR (a NaN argument now reads as a number, where the old
range check gave `#NUM!`) is moot: the engine produces no NaN number values.
