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
The 16 visibly-off cells of probes/forecast-ets-reprobe-2026-11 and the 40 of probes/forecast-ets-longer-series have
an owner: the web oracle gives `#N/A` on every FORECAST.ETS* call while the engine computes a number or a validation
error (decision 0221), so they stay known failures until a desktop-Excel backend exists.

## Context
T-3641 re-probed excel-graph on 2026-11-05; see statistics.md "FORECAST.ETS family: excel-graph never computes it".
Cells: probes/forecast-ets-reprobe-2026-11 Sheet1!I1:I8 and Sheet1!J1:J8. Desktop work is out of scope (decision 0266),
hence `defer`.
- Gap triage 2026-11-05 (main 3b2f779e, clean `make eval`): probes/forecast-ets-longer-series was owned by T-2425,
  dropped by decision 0266, and was left with no owner (`make unowned-off ALL=1` listed its 40 cells, the only unowned
  visibly-off cells of the run). Same root cause: Excel (web) is `#N/A` on every cell; the engine gives numbers
  (K1 `FORECAST.ETS(25,B1:B24,A1:A24)` is 148, K14 SEASONALITY 4, K11 CONFINT 0) and `#NUM!` on the out-of-range
  arguments (K35, K39-K42). K37 and K38 already match. statistics.md cites this case (K3, K8, K14, K19, K25, K30).
- probes/forecast-ets-longer-series.case.json (40): Sheet1!K1-K36, K39-K42

## Acceptance criteria
- [ ] If a desktop or web oracle ever returns numbers for the family, refill both cases and compare the cells
      (reprobe I1:I8 and J1:J8; longer-series K1:K42).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing or changing the ETS algorithm.
