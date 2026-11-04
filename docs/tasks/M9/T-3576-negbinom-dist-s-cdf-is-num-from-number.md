---
id: T-3576
title: "NEGBINOM.DIST's cdf is #NUM! from number_s 1E+13 at 28 failures"
milestone: M9
lane: functions-stat
status: in-progress
priority: P0
size: S
tier: opus
depends_on: [T-3553]
campaign: semantics
---

## Goal
`NEGBINOM.DIST(f, s, p, TRUE)` returns `#NUM!` wherever Excel does at a huge number_s, and the cut points are known:
probed and recorded in statistics.md.

## Context
- Found by T-3553's Gate A fill and pinned by probes-numeric/t3553-negbinom-huge-s (excel-graph, 2026-11-01).
  statistics.md has the entry: "NEGBINOM.DIST's cdf is #NUM! at a huge number_s (T-3553)".
- What the probe shows:
  - `#NUM!` at number_s 1E+13, 1E+14, 999999999999999, 1E+16 and 1E+20 with 28 failures, at p 0.1, 0.32 and 0.5.
  - 0 at number_s 1E+12 and below.
  - 0 at 0 and 14 failures with number_s 999999999999999.
  - The pmf (`FALSE`) and the legacy NEGBINOMDIST stay 0.
- The engine returns 0 at all seven `#NUM!` cells.
- Code: `crates/rustsheet-engine/src/functions/statistical/binom.rs`, the NEGBINOM.DIST kernel. Its cdf sums pmf terms in
  double-double, so Excel's cdf presumably takes another path that overflows or refuses there.
- The cuts are unknown: number_s between 1E+12 and 1E+13, failures between 14 and 28. Whether the cut depends on the
  product of the two is also unknown.
- **Attempt 1 (2026-11-04, decision 0056 time-box).** probes/t3576-negbinom-cdf-huge-s (89 cells, excel-graph)
  showed that neither cut is a threshold on number_s or failures. The error is Excel's incomplete beta's:
  `BETA.DIST(p,s,f+1,TRUE)` is `#NUM!` at the same points. Every `#NUM!` falls on TOMS 708 bratio's L140 branch
  (bup + bpser or bup + bgrat), except bfrac at number_s 1E+20.
  - On the bup + bpser cells, the log of bup's largest partial sum separates the probe: 0 up to 697.27, `#NUM!`
    from 703.57. That is not ln(DBL_MAX), and two swapped cells contradict it (A88 and A89 are `#NUM!` at 684 and
    804).
  - bup + bgrat fails from a log of 56.
  - statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s" and `docs/research/t3576/bup_partial_sum.py` hold
    the numbers. The probe's 37 cells with the wrong type are this task's: probes/t3576-negbinom-cdf-huge-s Calc!A5:A10, A18:A21,
    A34:A35, A42:A52, A60, A66:A71, A73, A76:A77, A82:A83, A88:A89. A75 is 1 ulp off.
  - Next: find which quantity Excel tests in bup, bpser and bgrat, for example by modelling bpser(a0, 1, x0) at a
    huge a0 in `bratio_excel.py` and checking where its terms overflow or a NaN appears. A threshold fitted to these
    points alone would be a fit (decision 0188 R4: hold-out and fresh sweep). BETA.DIST takes the same rule, so the
    fix belongs in `bratio/`, not `binom.rs`.

## Acceptance criteria
- [ ] A probe brackets the number_s cut and the failures cut, and tests whether the cut is a product or a sum of the two.
      statistics.md records the rule.
- [ ] The engine matches every probed cell, including probes-numeric/t3553-negbinom-huge-s (an ungated suite).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy of NEGBINOM.DIST at a large number of failures (T-3352).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 7 probes-numeric cells that no open task listed: 7 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3553-negbinom-huge-s.case.json (7): Calc!A1, A6-A7, A10-A11, A15-A16

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).
