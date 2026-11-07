---
id: T-4710
title: "CUMPRINC's underflowed principal at a result near 1E-285 and CUMIPMT's PMT·m half (e-T-4059 open misses)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3141]
---

## Goal
Find Excel's form for the two open misses of financial.md "CUMPRINC where `v^nper` underflows, at deep leading
exponents (T-4059)", and match it if a form is found.

## Context
- accuracy/CUMPRINC/e-T-4059 A119, `CUMPRINC(0.273951989085243,2756,134494.948128047,9,9,1)`: Excel gives
  -3.166759757937701E-285. T-3141's core is 962 ulps off it. Excel is 3.0E-14 off the math, the core 8.3E-14. This
  point has a leading exponent of 665 and a result near 1E-285; the other 199 points of the batch are exact.
- accuracy/CUMIPMT/e-T-3141 A41, `CUMIPMT(0.103414751928719,17312013078,200894.020467694,17312013077,17312013078,1)`. Excel gives -5128.51360859488 and ours is 4 ulps off whatever the
  principal's form is, so the miss is in `PMT·m`.
- Start from T-3141's kernel (cumulative.rs `underflowed_principal`) once T-3141 lands. Probe more points near
  results below 1E-280 with excel-probe.

## Acceptance criteria
- [ ] A119's form is evidenced and matched without moving any other exact point, or the probes that rule out the
  candidate forms are recorded in financial.md.
- [ ] The CUMIPMT case is probed in a case of its own and explained, or recorded as open with its candidates.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tiny-rate `PMT·m - CUMPRINC` cancellation (T-4390).
