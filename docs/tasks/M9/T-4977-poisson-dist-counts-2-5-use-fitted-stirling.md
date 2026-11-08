---
id: T-4977
title: "POISSON.DIST counts 2-5 use fitted Stirling-error constants instead of stirlerr_exact"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
POISSON.DIST's PMF at counts 2–5 matches Excel without per-count fitted Stirling-error constants.

## Context
`poisson.rs` (~24–44, 72–78) uses constants 13–34 ulp off the true stirlerr values, chosen to flip rounded exponents. Counterexample: `POISSON.DIST(4,4.8,FALSE)` (t2055-poisson-pmf-sweep A48) is 1 ulp off, and `stirlerr_exact` gives Excel's value.

Source: docs/research/excel-parity-findings-2026-11-07.md, "Audit findings in pre-existing code" item 1 (filed by T-4975). The audit's claims are unverified here: reproduce first.

## Acceptance criteria
- [ ] The counterexample cell passes.
- [ ] The fitted constants are removed or justified with a hold-out per decision 0188 R4; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Notes
Measured in T-4975: `stirlerr_exact` makes A48 exact but moves 48 other pmf cells, 9 gated (8 gen-m9 and ironcalc POISSON.xlsx D9); seven GAMMALN-based forms were tried and none fit. Needs a different cause (T-3247).
