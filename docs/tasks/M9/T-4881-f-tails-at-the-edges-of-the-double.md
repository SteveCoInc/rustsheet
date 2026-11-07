---
id: T-4881
title: "F tails at the edges of the double range: 0 where Excel's beta argument 1/(1+r) is subnormal, F.DIST.RT's subnormal result flushed, #DIV/0! at a subnormal x"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
F.DIST's cdf, F.DIST.RT and FDIST give Excel's exact 0 and Excel's error at the three double-range edges T-4857's
probe found, so the engine stops returning a number there.

## Context
T-4857's equivalence-class probe (probes-numeric/t4857-f-classes, excel-graph 2026-11-07; docs/research/t4857/
ANALYSIS.md section 6) found three rules (docs/excel-semantics/statistics.md "F tails at the edges of the double range
(T-4857)"):
- **The smaller beta argument goes subnormal.** Excel forms r = fl(df2/fl(df1 x)) (or its inverse) and the smaller
  argument fl(1/fl(1 + r)) (statistics.md "F's incomplete-beta arguments keep r >= 1"). Once r passes 1/DBL_MIN
  but is still finite, that argument is subnormal and Excel's tail on it is exactly 0, while the engine keeps the
  subnormal and returns a normal number. `F.DIST(2.2250738585072017E-306,1,400,TRUE)` is 0 in Excel and
  1.19E-153 in the engine (A1482); one double lower r overflows and both are `#NUM!` (A1481). The same on the upper
  side: `F.DIST.RT(2.27556092792985E+306,79,1.6063)` is 0, ours 5.27E-154 (A191), `FDIST(8.28578930344434E+302,
  216961.75,2)` is 0, ours 1.2E-303 (pair-tail A339).
- **F.DIST.RT and FDIST flush a subnormal result to 0**, as F.DIST's cdf does (T-4026): the `=(..)=0` companions are
  TRUE where the engine returns about 1E-309.
- **A subnormal x is `#DIV/0!`** for the cdf and both tails, as for the density (T-2586 r5): `F.DIST(EXP(-720),3,7,
  TRUE)`, `F.DIST.RT(EXP(-720),3,7)` and `FDIST(EXP(-720),3,7)` are `#DIV/0!`; ours `#NUM!` (the ratio-overflow
  rule fires first). A literal `1E-308` or `2^-1074` reads as 0 and gives 0 or 1 in both.
- Code: `fisher.rs` `f_dist_of`, `f_dist_rt_of`, `f_beta_ratio_overflows`; `arithmetic::flush_subnormal`.

Cells (probes-numeric/t4857-f-classes): A191, A375, A1341, A1379, A1398, A1482:A1485; C3187, C3197, C3207

## Acceptance criteria
- [ ] The cells above match Excel; the rest of t4857-f-classes and the t4026 and t2435 cases do not regress.
- [ ] A unit test per rule in `fisher.rs`; statistics.md's entry says "Ours: matches".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Where along r the 0 band ends on the lower side (A1485 is the last probed cell; a sweep belongs to a probe task).
