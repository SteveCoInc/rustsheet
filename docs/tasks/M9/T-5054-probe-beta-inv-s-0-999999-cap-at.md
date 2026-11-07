---
id: T-5054
title: "Probe BETA.INV's 0.999999 cap at beta 1 past the evidenced alphas and p (T-4958 follow-up)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Find out whether BETA.INV's 0.999999 at beta 1 (T-4958) holds for every alpha and every p up to 1/2 where p^(1/alpha)
rounds to 1, and for BETAINV too.

## Context
- T-4958 (`shape_one_quantile_cap`, functions/statistical/beta.rs) returns 0.999999 at beta = 1 and p <= 1/2 wherever
  p^(1/alpha) rounds to 1, up to the sqrt(DBL_MAX) cut on alpha. The evidence is only p = 0.1 at alpha 5.62E+16 to
  1E+18 (probes/t4713-beta-inv-beta-above-1-cut F12:F17), and every cell is BETA.INV (excel-fidelity review MINOR on
  #4040).
- statistics.md "BETA.INV at beta 1 caps at 0.999999 where its quantile rounds to 1 (T-4958)".
- Use the excel-probe skill: predictions (the engine's values) committed before the fill.

## Acceptance criteria
- [ ] A probe case gives BETA.INV(p, a, 1) at a = 1E+20, 1E+100 and 1E+150, with p = 1E-300, 1E-15, 0.1 and 0.5, and
      BETAINV at two of those points, plus the controls F11 and F12 of t4713.
- [ ] statistics.md's T-4958 entry is updated with the result: the cap confirmed over the range, or the bound or
      p-dependence found, with the engine changed in a follow-up task if needed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 0.999999 at a tiny beta (T-4700).
