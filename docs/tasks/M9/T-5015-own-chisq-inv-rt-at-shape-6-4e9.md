---
id: T-5015
title: "Own CHISQ.INV.RT at shape 6.4E9 (t3409-r4-gamma-inv-search-sweep Calc!A234): Excel is 2.7E-4 off the math, so R3 or a fit"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
probes-numeric/t3409-r4-gamma-inv-search-sweep Calc!A234 `CHISQ.INV.RT(1.55608746068665E-92,12890000000)` is settled,
either recorded as a decision 0188 R3 point (MATH-OVER-EXCEL, as T-4205 treats Calc!A226 at shape 1E9 or more) or
matched by a fit, so `make exit-owners MILESTONE=M9` no longer lists it.

## Context
The 2026-11-07 evening steward run (`make eval` on c709e9d2) found `make exit-owners MILESTONE=M9` listing this cell
under the CHISQ.INV.RT rate with no open M9 owner. Excel gives 12896697822.579903, ours 12893270849.57774, which equals
the 50-digit math sidecar: Excel's relative error is about 2.7E-4, above R3's 1E-4. The shape is df/2 = 6.445E9, so
T-4205 (shapes below 1E9) and T-4206 (the search's type at its step) leave it out, as T-4205 leaves out Calc!A226
and t3409-r4-gamma-inv-search-sweep-wide Calc!A114. Code: the gamma inverse search in functions/statistical (T-3409).

## Acceptance criteria
- [ ] Calc!A234 is either an R3 point under decision 0188 (with the record a new case needs) or bit-exact, with the
  R4 guards of reviewer-excel-fidelity check 7 if it is fitted.
- [ ] `make exit-owners MILESTONE=M9` no longer lists it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other cells of the sweep, which T-4205 and T-4206 own.
