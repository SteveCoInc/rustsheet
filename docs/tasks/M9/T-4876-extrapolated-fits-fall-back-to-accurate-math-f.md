---
id: T-4876
title: "Extrapolated fits fall back to accurate math: F density Stirling error past df 56 and the Student-t df bands (T-3364 rest)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3364]
---

## Goal
The F density's `STIRLING_ERROR` table and the Student-t df bands apply only inside the inputs they were measured on,
and fall back to accurate math outside them (decision 0188 R4.3).

## Context
- Split from T-3364 (2026-11-07), which moved GAMMA.INV's huge-shape closed-form branch to the math (decision 0337).
  These two parts remain from the owner's 2026-10-31 audit of extrapolated fits (decision 0188):
  - statistical/f_density.rs `STIRLING_ERROR` table: used past df 56, beyond its measured cells.
  - statistical/student_t.rs df bands (`LOG_GAMMA_BETA_MIN` 6.5, `LOG_GAMMA_BETA_MAX`, `SERIES_MAX_DF`,
    `ASYMPTOTIC_MIN_SHAPE`).
- Use each site's recorded `FITTED(T-XXXX, n_cells, range)` tag (T-3361). T-3354 already reworked the F density at a
  tiny x; build on it.

## Acceptance criteria
- [ ] Each piece above applies only inside its recorded `FITTED` range; outside it the code computes the accurate value
      (an mpmath-checked unit test on each side of each boundary).
- [ ] A fresh log-uniform sweep over each affected function's domain (R4.2) shows the Gate A envelope (R2).
- [ ] No gated cell regresses beyond decision 0083's allowance.
- [ ] The math path keeps directive 5's caps (bounded iterations, no panic).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-validating the in-range form selectors (T-3365).
- GAMMA.INV's huge-shape branch (T-3364, T-4205).
