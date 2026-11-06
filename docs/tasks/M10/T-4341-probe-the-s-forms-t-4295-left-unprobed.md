---
id: T-4341
title: "Probe the [s] forms T-4295 left unprobed: [ss] m, a second h/m token, a date or elapsed token before [s], [s].0 m and s [s] m"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4295]
---

## Goal
Excel's TEXT output for the `[s]` format shapes that T-4295's rewrite (`move_elapsed_second`,
crates/rustsheet-engine/src/formatter/elapsed.rs) handles without evidence is probed and recorded. The engine then
matches it, or the gap is filed.

## Context
- T-4295 (PR #3308) found that `[s]` before an `h` or `m` token hands that token the elapsed count: `[s] m` on
  1.7/24+0.3/1440 is "18 26640". Its probes, probes/t4295-elapsed-with-month and -b, cover that rule.
- The review of #3308 (design and excel-fidelity, MINOR) listed shapes the rewrite also touches but nobody probed:
  - `[ss] m`, and a second `h` or `m` token after the first (`[s] m m`, `[s] h m`). The rewrite moves only the
    first target. number-formats.md "Elapsed-time tokens" marks both UNVERIFIED.
  - A date token or an elapsed token before `[s]` (`d [s] m`, `mmm [s] h`, `[h] [s] m`). The rewrite fires there,
    and only a clock token before `[s]` stops it.
  - `[s].0 m`, which the rewrite turns into `s.0 [m]`.
  - `s [s] m`: the unit test asserts `#VALUE!`, and the semantics entry marks it UNVERIFIED.
- Follow `docs/research/t4295/make_probe.py` for the probe shape, at a few serials: 1.7/24+0.3/1440, 45000.123, and
  one below 1.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes holds each shape above, with oracle values (excel-probe skill).
- [ ] number-formats.md "Elapsed-time tokens" states what Excel does for each shape, and drops the UNVERIFIED marks the
      probe settles.
- [ ] The engine matches every probed cell, or each mismatch is filed as a task and recorded in the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Elapsed tokens without `[s]`.
