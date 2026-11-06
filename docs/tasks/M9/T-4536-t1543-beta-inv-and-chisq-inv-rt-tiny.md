---
id: T-4536
title: "t1543 BETA.INV and CHISQ.INV.RT tiny-p cells left by T-3386: three #NUM! against Excel numbers and two Excel-target digits"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The five probes-numeric/t1543-inverse-cdf-tiny-p cells T-3386 listed but could not resolve match Excel (directive 1)
or are accepted under decision 0056's time-box (decision 0267).

## Context
- From T-3386 (decision 0311), checked 2026-11-06 against the committed math sidecar:
  - A89 `BETA.INV(1-1E-16,1000,1)`: Excel 1.0, engine `#NUM!`.
  - A91 `BETA.INV(1E-300,0.5,100)` and A92 `BETA.INV(1E-200,0.5,100)`: Excel 2.7050408442680296E-151 for both, engine
    `#NUM!`; the true quantile (about 7.9E-603) underflows, and the sidecar lists both as having no single value.
    statistics.md calls Excel's BETA.INV "erratic where the shape is extreme". T-4231 covers their sidecar check.
  - A79 `BETA.INV(1E-300,5,2)`: Excel 6.988271195153886E-61 is 1.06E-9 relative off the math, the engine
    6.988271187715847E-61 is on it. Excel is within G, so Excel is the target.
  - A43 `CHISQ.INV.RT(1E-300,50)`: Excel 1592.6841019848946 is 6.0E-12 off the math, the engine 1592.6841019943863 on
    it. Excel is the target.
- An error cell never passes through an R3 mark (decision 0188 R3).

## Acceptance criteria
- [ ] A89, A91 and A92 return a number (Excel's) or are accepted under the time-box, with the reason in the Outcome.
- [ ] A43 and A79 are within half a unit of Excel's 15th significant digit, or are accepted under the time-box.
- [ ] No passing cell regresses; `make check` green; reviewers approve.

## Out of scope
- R3 marks: none of these cells qualifies.
