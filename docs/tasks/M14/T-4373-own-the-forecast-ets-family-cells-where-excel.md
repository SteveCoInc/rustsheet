---
id: T-4373
title: "Own the FORECAST.ETS family cells where excel-graph gives #N/A (re-probe 2026-11-05)"
milestone: M14
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The 16 visibly-off cells of probes/forecast-ets-reprobe-2026-11 have an owner: the web oracle gives `#N/A` on every
valid FORECAST.ETS* call while the engine computes a number (decision 0221), so they stay known failures until a
desktop-Excel backend exists.

## Context
T-3641 re-probed excel-graph on 2026-11-05; see statistics.md "FORECAST.ETS family: excel-graph never computes it".
Cells: probes/forecast-ets-reprobe-2026-11 Sheet1!I1:I8 and Sheet1!J1:J8. Desktop work is out of scope (decision 0266),
hence `defer`.

## Acceptance criteria
- [ ] If a desktop or web oracle ever returns numbers for the family, refill the case and compare the cells I1:I8 and J1:J8.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing or changing the ETS algorithm.
