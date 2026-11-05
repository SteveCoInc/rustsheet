---
id: T-4335
title: "Probe [s] before h or m after a date token, with a fraction, and [ss] (after T-4295)"
milestone: M10
lane: functions-text
status: dropped
priority: P3
size: S
tier: sonnet
depends_on: [T-4295]
---

## Goal
Excel's output for the `[s]` forms T-4295 did not probe is known and recorded, and the engine matches it.

## Context
T-4295 (PR #3308; docs/excel-semantics/number-formats.md, "Elapsed-time tokens") found that `[s]` before an `h` or
`m` token hands its elapsed count to that token (`[s] m` gives "18 26640" for 1.7/24+0.3/1440), and that a clock
token before `[s]` stops the move (`s [s] m` stays #VALUE!). The engine's rule is formatter/elapsed.rs
`move_elapsed_second`. Its review (MINOR, excel-fidelity) listed forms nobody probed:
- a date token before `[s]`: `d [s] m`, `mmm [s] h`, `yy [s] h` (the engine still moves the count there);
- a fraction on the moved seconds: `[s].0 m`, `[s].00 h`;
- `[ss] m` and `[ss] h`, and a second hour or minute token after `[s]` (`[s] m h`), marked UNVERIFIED in
  number-formats.md.
Use the excel-probe skill; the T-4295 generator is docs/research/t4295/make_probe.py.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers every form above in both date systems, filled from excel-graph.
- [ ] The engine matches every new cell, or each mismatch has an owner task; number-formats.md records the results
      and drops the UNVERIFIED marks the probe settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Elapsed tokens without `[s]` (T-4295 and earlier).

## Outcome
Dropped as a duplicate: T-4341 (filed from PR #3308's second review round) probes the same `[s]` forms, plus `s [s] m`.
