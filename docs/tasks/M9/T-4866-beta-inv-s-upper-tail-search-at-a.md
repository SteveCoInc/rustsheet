---
id: T-4866
title: "BETA.INV's upper-tail search at a tiny alpha and a large beta lands far off the root"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
`BETA.INV(1-1.10491974750915E-14,0.0050070029475036,171.171095424379)` returns Excel's 0.12939405629301814 (the
50-digit math agrees), not the engine's 2.925059559426143E-5.

## Context
- Found by T-3351's fresh sweep (probes-numeric/t3351-beta-inv-sweep A200, excel-graph). It is the only point of
  that sweep outside the Gate A envelope that is a number on both sides. The quantile does not round to 1, so
  T-3351's code path (`beta.rs` `upper_quantile_near_one`) never runs there.
- PR #2573's analysis (2026-10-31, on an older `beta.rs`): at these shapes the upper-tail incomplete beta the
  search crosses returns exactly 0 or 1 over a wide band, so the refinement moves statrs' correct start off the
  root. Re-check it against the current search (`beta_quantile`, `bratio::beta_search_tails` with p above 1/2).
- Code: crates/rustsheet-engine/src/functions/statistical/beta.rs `beta_quantile`, bratio/mod.rs.

## Acceptance criteria
- [ ] Sweep A200 is within 15 digits of Excel; no other cell of the BETA.INV probes and sweeps moves away from
      Excel (`make eval-check`).
- [ ] The cause is recorded in docs/excel-semantics/statistics.md (or the engine note, if it is ours alone).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Quantiles at an end of the doubles (T-4865).
