---
id: T-4786
title: "Math sidecars for t3444's BINOM.DIST over a BINOM.INV result cell and its two not-converged cdf differences at n = 1E6 (17 counted M9 probe cells)"
milestone: M9
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every cell of probes/t3444-binom-range-inv-large-n has a 50-digit math value where the math is defined. That covers
the Inv sheet's `BINOM.DIST(A<k>,n,p,TRUE)` and `BINOM.DIST(A<k>-1,n,p,TRUE)` cells, whose first argument reads a
BINOM.INV result, and the Range sheet's BINOM.DIST differences that `math` now skips as "not converged". Afterwards
decision 0188 R1(c) counts none of the case's cells just because it has no math value (decision 0325).

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main c18fafcce, excel-graph). `make unowned-off ALL=1` lists 17
  visibly-off cells with no open owner, all in this case. The M9 probe row counts each one under R1(c) because it has
  no math value, not because it is outside an envelope. T-3740 and T-3741, which owned the case, are done.
- Cells this task owns:
  - probes/t3444-binom-range-inv-large-n.case.json (15): Inv!C36, C50, C54, C68, C72, D31, D36, D49, D50, D52, D54, D67, D68, D70, D72
  - probes/t3444-binom-range-inv-large-n.case.json (2): Range!B49, B53

  T-4787 owns the same 17 cells as Phase 1b evidence once they have math values, together with the case's other
  unowned last-digit cells.
- Why the Inv cells have no math: the sidecar (tools/src/rustsheet_tools/accuracy/eval_math.py, `LiteralBook`) reads a
  referenced formula cell only when its formula is arithmetic over literals. Column A is `=BINOM.INV(n,p,alpha)` and
  column B is `=CRITBINOM(...)`. Both already have math values in the sidecar, and both are exact integers. The math
  file lists no C or D cell, not even as skipped: Inv has 126 A and 126 B math cells. The Out of scope of both T-3924
  and T-3993 left out "arguments that read formula results (t3444's `BINOM.DIST(A1-1,...)` over BINOM.INV)".
- Why B49 and B53 have no math: `skipped` says `not converged` for
  `=BINOM.DIST(1100,1000000,0.001,TRUE)-BINOM.DIST(999,1000000,0.001,TRUE)` and
  `=BINOM.DIST(1000000,1000000,0.001,TRUE)-BINOM.DIST(999,1000000,0.001,TRUE)`. The case has 89 such skips on Range,
  most of them at n = 1E9 and above. BINOM.DIST's cdf definition (tools/src/rustsheet_tools/accuracy/defs/discrete.py)
  goes through mpmath's incomplete beta, and near the mean at a large n that 2F1 series does not converge (mpmath's
  `betainc` raises `hypsum() failed to converge` at n = 1E6, k = 939). For exactly this reason defs/binom_range.py
  already sums the pmf outward from k, with each term charged to the work budget (T-3741).
- Triage spot check (mpmath at 60 digits, pmf summed from 0, p read as the decimal literal):

  | cell | formula, k | Excel - math | ours - math |
  |---|---|---|---|
  | Inv!C36 | `BINOM.DIST(A36,100000,0.001,TRUE)`, k = 95 | -2.6E-13 rel | -7.9E-15 rel |
  | Inv!C54 | `BINOM.DIST(A54,1000000,0.001,TRUE)`, k = 983 | -9.7E-13 rel | +9.6E-16 rel |
  | Inv!D67 | `BINOM.DIST(A67-1,10000000,0.001,TRUE)`, k = 9999 | -1.3E-12 rel | -1.6E-15 rel |
  | Inv!D70 | `BINOM.DIST(A70-1,10000000,0.001,TRUE)`, k = 9528 | -3.2E-13 rel | -4.5E-13 rel |

  Excel and the engine are both within 1E-9 of the math on every one. The sign is T-3961's regime: at p below 0.01
  and a huge n, Excel is always lower. So once math is attached the 17 cells should move to Phase 1b evidence
  (decision 0325) instead of being counted. The Range sheet's BINOM.DIST.RANGE twins of B49 and B53 (A49, A53)
  already have math and are already Phase 1b evidence: Excel is 1.19E-12 relative off the math there, ours 8.8E-16.

## Acceptance criteria
- [ ] probes/t3444-binom-range-inv-large-n.math.json gives a math value to every Inv C and D cell whose A cell has
      an integer math value. The argument is read as that integer, or that integer minus 1 for the D cells.
- [ ] The rule for when a formula-result argument is read (an exact integer math value, nothing else) is written in
      eval_math.py's module doc and in SPEC-0029 "Math references for eval cells".
- [ ] The rule has unit tests in tools/tests: a cell that is read, cells that are refused (non-integer math, no math),
      and `A1-1` over a cell that is read.
- [ ] Range!B49 and B53 have math values, and so does every other `not converged` BINOM.DIST point of the case that
      the work budget allows.
- [ ] If the BINOM.DIST cdf definition changes, its fingerprint changes with it:
  - `make eval-math-check` is current afterwards;
  - every Gate A table that hashes the definition is regenerated in this PR;
  - the Outcome quotes the envelope result.
- [ ] After a full `make eval`, `make unowned-off ALL=1` lists none of the 17 cells.
- [ ] The Outcome quotes the report's R1(c) line and the M9 probe column (counted and visibly-off totals), before and
      after.
- [ ] If any of the 17 cells still counts (Excel more than 1E-9 off the math), the Outcome names it with an open
      `campaign:` owner, filed if none exists.
- [ ] `make eval-baseline` records nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change. BINOM.DIST's own last digits are T-4787's, after T-3961's model.
- Formula-result arguments in other cases. If `math --check` shows any, name them in the Outcome as follow-ups.
