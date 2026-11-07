---
id: T-2798
title: "TEXT: AM/PM before an elapsed token uses the 12-hour clock for it"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(1.5,"AM/PM [h]")` prints "PM 12" like Excel: when an AM/PM or A/P marker comes first, the elapsed hour uses the 12-hour clock.

## Context
probes/text-elapsed-two-tokens.case.json S!A13 (excel-graph "PM 12", engine "PM 36"); dates.md "Negative serials under an explicit negative section". Discovered in T-1366. Only the marker-first order is valid; the marker after an elapsed token is #VALUE! (done).

Exit owners (decision 0096; the 2026-10-27 planner run names each cell the `.case.json` line above holds):
- probes/text-elapsed-two-tokens S!A13

## Acceptance criteria
- [ ] S!A13 passes and leaves eval/baseline/probes.txt; probe other elapsed tokens (`AM/PM [mm]`, `[s]`) first.
- [ ] `make check` green; reviewers approve.


## Outcome
Already satisfied on `origin/main`: `probes/text-elapsed-two-tokens` has no `F` line in
`eval/baseline/probes.txt` (all 14 cells pass), and `TEXT(1.5,"AM/PM [h]")` prints "PM 12" (also
pinned in `formatter/test/test_general.rs`). No code change, eval delta 0. Note: a 29 h serial
(`AM/PM [hh]` on 1+5/24) prints "AM 12" in the engine; no oracle cell covers it, so it is
unverified against Excel and not changed here.
