---
id: T-4974
title: "Fill the 2026-11-07 probe suggestions (PR 4002) with excel-graph"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The 24 draft probe cases of PR #4002 (docs/research/probe-suggestions-2026-11-07, groups G01-G24) are in the probes
suite with excel-graph values, scored against the engine, and every visibly-off cell they add has an owner task.

## Context
PR #4002 lists probes that would confirm or falsify rules the 2026-11-07 Excel-parity fixes inferred rather than
measured (gamma Q-to-P switch and split complement, AS 109 stops, F back-transform, Student-t stops, LOGNORM density,
Loader store chain, SKEW/KURT x87 rounding (T-4328), IRR caps, BETA.INV boundaries, T.DIST at df 1000). The cases are
copied as `eval/suites/probes/cases/t4974-<group>.case.json` (the `val-` prefix dropped), unchanged otherwise.

## Acceptance criteria
- [x] The 24 cases have excel-graph `.oracle.json` and Gate A `.math.json` sidecars.
- [x] Their failing cells are baselined (`make eval-baseline`) and every visibly-off cell has an owner task.
- [x] The per-group outcome (which inferred rules Excel confirms or falsifies) is in the Outcome note.
- [x] `make check` green; reviewers approve.

## Out of scope
- The merged-range scenarios M1-M6: they are operation scripts, not cases (`sheet_ops` has no unmerge, clear,
  paste, autofill or edge op); excel-graph cannot run copy, autofill or edge navigation. SPEC-0036 (T-4722) and its
  tasks T-5044, T-5045, T-5047 and T-5048 own covered-cell content.
- Fixing the engine where a probe falsifies a rule (owner tasks).

## Outcome
- The 24 cases are `probes/t4974-<group>.case.json` with excel-graph sidecars (24 filled, no failures) and 23 Gate A
  math sidecars (the moments case has no single-call cell). The merged-range scenarios M1-M6 are not cases (out of scope).
- Against ours: 5,775 checked cells, 3,219 known-failure lines (mostly last digit: main lacks the prototype fixes the
  predictions came from), 325 visibly-off cells outside Phase 1b, owned by T-4994, T-4995, T-4996, T-4997 and T-4998. The visible-parity lock
  records 915 visibly-off cells for these cases: the other 590 are Phase 1b evidence under R1(c) (ours and Excel within
  1E-9 of the math), which the owner gate does not count.
  No other suite changed. Rescored on main 51dd32f9d (after T-4713, T-3469, T-1942 and the merged PR #4002 / #4005):
  every t4974 known-failure and visibly-off line is unchanged, so none of those fixes closes a cell of these probes.
  Rescored again on main db7cd416a: T-1871 (F quantiles at a tiny p) and T-3351 (BETA.INV near 1) close 13 last-digit
  cells (9 of t4974-beta-f-inv-endpoint-saturation, 4 of t4974-f-inv-back-transform); the visibly-off cells are unchanged.
  PR #4005's addendum (no residual flush for BETA.INV) agrees with the G22 result.
- Against the prototype (cells bit-exact / within 5e-15 / visibly off or a different error):
  confirmed outright: moments 204/0/0, LOGNORM density 148/0/0, gamma split complement 205/50/0, AS 109 accuracy
  exponent 322/302/0, F AS 109 cap 139/36/0, Student-t quarter stop 105/7/0 and tiny centre 111/36/0, T.DIST df 1000
  83/115/0, NORM.S.INV 82/144/0 (research data). Disagree visibly in some cells (a rule falsified at least there): BETA.INV DBL_MIN residual 30/12/90 (Excel
  does not flush), tiny-shape `#NUM!` boundary 192/0/80, endpoint saturation 228/23/29, large-shape round trip 670/156/35,
  gamma small-x prefactor 163/142/55, far upper tail 21/72/17, F tiny tail 75/6/9, F back-transform 286/86/12, Student-t
  joint stop 92/11/7, IRR 51/8/4, Loader 65/166/3, unit-x .5 220/151/4, small-beta tails 45/14/1. At df 3 Excel's
  CHISQ.INV.RT(p) equals CHISQ.INV(1-p) bit for bit from .9999 + 2E-6 to .99999 and differs from it at .9999; the rows
  within 4 ulps of .9999 have no comparator cell, so the cut's side is unmeasured (statistics.md "The PR 4002
  validation probes").
