---
id: T-4392
title: "Owner ruling on R3 for GAMMA.DIST's tiny-ratio density, which Excel's log form underflows to 0 (T-3925 follow-up)"
milestone: M0
lane: harness
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's `GAMMA.DIST(x,2,1,FALSE)` = 0 at x of 1E-300, 1E-250 and 1E-200, where
the math is about x (relative error 1), is a decision 0188 R3 (`MATH-OVER-EXCEL`) case. Until the owner rules, the
engine keeps matching Excel.

## Blocked because
Blocked because: needs an owner ruling (decision 0188 R3: "no agent, steward included, adds a case on its own").

## Context
- statistics.md "GAMMA.DIST's density below shape 3 is a log-space form (T-2753, T-2852)": where y = x / beta is at most
  about 1E-15, Excel's density is EXP(a*LN(y)-y)/GAMMA(a)/x. At shape 2 and y below about 1E-162, EXP(2*LN(y))
  underflows to 0, so the density is 0 though the true value, y e^-y, is a normal double.
- Evidence (excel-graph; math from T-3925's regenerated sidecars): probes/t2753-gamma-density-ratio-sweep A13
  (`GAMMA.DIST(1E-300,2,1,FALSE)`, math 1.0E-300, at the envelope's floor), A14 (`(1E-250,2,1,FALSE)`, math 1.0E-250) and
  A15 (`(1E-200,2,1,FALSE)`, math 1.0E-200): Excel 0, relative error 1 (past R3's 1e-4).
- The T-3925 Gate A table has no such point (0 failing); no other GAMMA.DIST, CHISQ.DIST or CHISQ.DIST.RT cell of
  probes or probes-numeric is a number more than 1e-4 off the math.
- The owner's options: (a) keep matching Excel, so this is no R3 case (write it into the T-2753 entry); (b) an R3 case
  for the underflowing log form, with a decision record and an implementation follow-up.

## Acceptance criteria
- [ ] A decision record carrying the owner's ruling, or the owner's "keep matching Excel" written into the
      statistics.md T-2753 entry.
- [ ] If (b): a follow-up implementation task, the region's evidence committed first (decision 0188 R3).

## Out of scope
- Shapes other than 2 and the T-3219 shape-50 band.
