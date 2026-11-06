---
id: T-4277
title: "Owner ruling on R3 for GAUSS between 1E-15 and 1E-8, where Excel's Phi(z)-0.5 cancels (T-3783 follow-up)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's GAUSS(z) for 1E-15 < |z| < 1E-8, where it is the cancelled Phi(z) - 0.5
in doubles, is a decision 0188 R3 (`MATH-OVER-EXCEL`) case. Until the owner rules, the engine keeps matching Excel.


## Context
- statistics.md "GAUSS of a tiny argument is x times phi(0) (T-2255)": GAUSS keeps its leading term z*phi(0) up to
  |z| = 1E-15, and from about 4E-15 it is Phi(z) - 0.5 in doubles. Its absolute error near 2^-54 is about
  1.4E-16/|z| relative, so it exceeds R3's G = 1e-4 below about |z| = 1.4E-12.
- Evidence: gen-m9/146b46e74f2dac3b A46, `GAUSS(4E-15)`: Excel 1.5543122344752192E-15, math 1.5957691216057308E-15
  (2.6% off). gen-m9/68d8ffb69ff739d8 A262, `GAUSS(1E-10)`: Excel 3.9894310077670525E-11 (2.1E-6 off, inside R3).
  Excel-graph, 2026-10-22.
- The engine matches Excel at these cells (directive 1). T-3783's Gate A table leaves the region out: a GAUSS point is
  redrawn while 1E-15 < |z| < 1E-8. So the gate does not depend on this ruling.
- The owner's options:
  - (a) keep matching Excel, so this is no R3 case;
  - (b) an R3 case over a bounded region of z, with a decision record.

## Acceptance criteria
- [x] A decision record carrying the owner's ruling, or the owner's "keep matching Excel" written into the
      statistics.md T-2255 entry.
- [x] n/a (decision 0300 keeps Excel): If (b): a follow-up implementation task with its region, and the region's evidence committed first
      (decision 0188 R3).

## Out of scope
- Changing the engine before the ruling.
- ERF's two-argument cancellation (T-3794).

## Outcome
- Owner ruling 2026-11-06, decision 0300: keep matching Excel. This is not an R3 case: no `MATH-OVER-EXCEL`
  entry, no R3 mark, no engine change (the engine already matches Excel here).
- Eval delta: none.
