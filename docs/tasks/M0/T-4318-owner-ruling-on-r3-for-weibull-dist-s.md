---
id: T-4318
title: "Owner ruling on R3 for WEIBULL.DIST's density at x = 0, which Excel gives as 0 for every shape (T-3785 follow-up)"
milestone: M0
lane: harness
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's `WEIBULL.DIST(0,alpha,beta,FALSE)` = 0 for alpha = 1, where the math is
1/beta, is a decision 0188 R3 (`MATH-OVER-EXCEL`) case, and what alpha < 1 (an unbounded density, where Excel also
gives 0) is. Until the owner rules, the engine keeps matching Excel.

## Blocked because
Blocked because: needs an owner ruling (decision 0188 R3: "no agent, steward included, adds a case on its own").

## Context
- statistics.md "Density at 0" (T-2035, gap triage 2026-10-19) and "WEIBULL.DIST's density at x = 0 still computes
  beta^alpha (T-3312)": Excel's density at x = 0 is 0 for every probed shape, after the beta^alpha errors.
- Evidence (excel-graph; math from T-3785's sidecars):
  - alpha = 1, number against number, relative error 1 (past R3's 1e-4): probes-numeric/t1048-weibull A16
    (`WEIBULL(0,1,1,FALSE)`) and B16 (`WEIBULL.DIST(0,1,1,FALSE)`), math 1; probes-numeric/t1049-weibulldist A27
    (`WEIBULL.DIST(0,1,3,FALSE)`), math 1/3.
  - alpha < 1, the density is unbounded, so no math value: t1048-weibull A17 and B17 (alpha = 0.5), t1049-weibulldist
    A28, probes/t3312-xirr-xnpv-inner-calls A6. The T-3785 definition records no math value there.
- The engine matches Excel at these cells (directive 1). The T-3785 Gate A table has no such point.
- The owner's options:
  - (a) keep matching Excel, so this is no R3 case;
  - (b) an R3 case for alpha = 1 (1/beta), with a decision record; alpha < 1 has no number to follow (`#NUM!` would be
    a new error, which R3 does not allow: Excel's errors are still matched).

## Acceptance criteria
- [ ] A decision record carrying the owner's ruling, or the owner's "keep matching Excel" written into the
      statistics.md "Density at 0" entry.
- [ ] If (b): a follow-up implementation task, the region's evidence committed first (decision 0188 R3).

## Out of scope
- Changing the engine before the ruling.
- F.DIST's and CHISQ.DIST's densities at 0 (Excel's `#NUM!` and the df = 2 values agree with the math's limits).
