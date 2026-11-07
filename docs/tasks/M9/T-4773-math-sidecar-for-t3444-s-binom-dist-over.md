---
id: T-4773
title: "Math sidecar for t3444's BINOM.DIST over BINOM.INV cells and its not-converged large-n cdf differences (17 visibly-off cells with no math value)"
milestone: M9
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The 17 visibly-off BINOM.DIST cells of the T-3444 probe listed under Context get a 50-digit math value in the
case's math sidecar. Decision 0188 R1(c) then scores them against the math: as Phase 1b evidence, or as a counted
cell with a real fit owner (decision 0325).

## Context
- Gap triage 2026-11-07 (`make eval` on origin/main b9ae5216, excel-graph). `scripts/exit_owners.py --all-off` gives
  these 17 cells as the only visibly-off cells with no open owner. `--milestone M9` counts them too. T-3447 listed
  them, and it is done. Its hand-off T-3774 lists only the t3352 cells, so nothing has owned them since.
- R1(c) counts them only because they have no math value: none is in the sidecar's `sheets` or `skipped` lists. Under
  decision 0325 that makes them this task's. Do not give them to a `visible` or `accuracy` fit task before the math is
  attached. There are two shapes.
  - Inv!C and Inv!D, `BINOM.DIST(A<r>,n,0.001,TRUE)` and `BINOM.DIST(A<r>-1,n,0.001,TRUE)`. Here `A<r>` is a
    `BINOM.INV` or `CRITBINOM` cell, and Excel's value of it equals the sidecar's math value of it (an integer) on every
    row below. `LiteralBook` reads a referenced formula cell only when its formula is arithmetic over literals, so
    these calls get no entry at all. T-3924's Out of scope names exactly this shape ("t3444's `BINOM.DIST(A1-1,...)`
    over `BINOM.INV`"), and no task took it up.
  - Range!B49 and B53, `BINOM.DIST(s2,1000000,0.001,TRUE)-BINOM.DIST(999,1000000,0.001,TRUE)`. These are skipped as
    `not converged` (dps 60 and 80 differ). The same reason skips 89 Range!B cells of the case, every one at n 1E4 or
    more. mpmath's `betainc` hypsum also fails at 7248 bits for `BINOM.DIST(95,100000,0.001,TRUE)`.
- The cells (Excel, then ours; relative error against a 50-digit pmf-recurrence sum `sum_{i<=k} C(n,i) p^i q^(n-i)`
  where computed for this triage):
  - probes/t3444-binom-range-inv-large-n.case.json (17): Inv!C36, C50, C54, C68, C72, D31, D36, D49, D50, D52, D54, D67, D68, D70, D72, Range!B49, B53
  - Inv!C36 `BINOM.DIST(95,100000,0.001,TRUE)`: 0.3311016442530485 / 0.33110164425313326. Math
    0.33110164425313579253: Excel -2.6E-13, ours -7.6E-15.
  - Inv!D49 `BINOM.DIST(999,1000000,0.001,TRUE)`: Excel -1.2E-12, ours +9.1E-16. Inv!C72 (n 1E7): Excel -1.0E-12,
    ours -5.3E-16. Inv!D52 (cdf 8.6E-7): Excel -3.0E-13, ours -1.3E-13. Inv!D70 (cdf 1.0E-6): Excel -3.1E-13, ours
    -4.4E-13.
  - Range!B49: 0.5033486554156034 / 0.5033486554150031. Math 0.50334865541500345255: Excel +1.2E-12, ours -7.7E-16.
  So Excel is always the side far from the math here, at p 0.001 with n from 1E5 to 1E7. That is the regime of T-3961
  (BINOM.DIST cdf at p below 0.01 and a huge n). Each Excel value is within 1E-9 of the math, so once the math is
  attached these cells are expected to be Phase 1b evidence (R1(c)), not counted.
- Two lines of approach, both inside the accuracy generator (`rustsheet-accuracy math`):
  - Resolve a single-cell argument through the sidecar's own math value of that cell, when the cell is a one-call Gate A
    cell whose math is an exact integer that Excel's value equals. `eval_math.LiteralBook`, `fitlab.sweep.cell_number`
    and `parse_arg(cell_refs=True)` are where references resolve today. Reuse them; do not add a second resolver
    (directive 3).
  - Give the BINOM.DIST definition (tools/src/rustsheet_tools/accuracy/defs/, the Gate A BINOM.DIST definition and
    binom_range.py) a convergent route at a large n. One example is the exact pmf recurrence summed from the nearer
    tail at 50 digits, for k up to a bound that keeps the generator's time limit. That route must agree with `betainc`
    wherever both converge, and the Gate A tables must not move.
- Code and rules: tools/src/rustsheet_tools/accuracy/eval_math.py (module doc rules; `NOT_CONVERGED`, `REGENERATE`),
  tools/src/rustsheet_tools/fitlab/sweep.py, docs/specs/SPEC-0029-gate-a-accuracy-gate.md (the sidecar rules to amend),
  `make eval-math-check`. T-3924 and T-3993 (done) are the precedents for widening the sidecar.
- Related, not overlapping: T-4774 owns the case's other 228 failing cells (BINOM.DIST's own last digits and the
  BINOM.DIST.RANGE rows equal to its cdf difference). T-3961 owns the p < 0.01 cdf fit on the t3352 and t3960 cases.
  T-4746 owns the past-mean Range!A rows.

## Acceptance criteria
- [ ] The t3444 math sidecar has a math value for each of the 17 cells above, regenerated with the documented
      command. Any cell still without one is listed under `skipped` with a reason, and named in the Outcome.
- [ ] SPEC-0029 (or the eval_math module doc it points to) states the new rule for a reference to a formula cell, or
      the new BINOM.DIST route, and unit tests in tools/tests cover it: a resolved integer cell, a refused non-exact
      cell, and agreement with `betainc` where both converge.
- [ ] `make eval-math-check` is current. No Gate A table point moves, and `make eval-baseline` records nothing.
- [ ] The Outcome quotes report.md's R1(c) line before and after. It also gives the result of
      `scripts/exit_owners.py --all-off` for these 17 cells: each is Phase 1b evidence, or is named in an open fit task
      with a `campaign:` key (T-3961 when Excel and the engine are not both within 1E-9 of the math).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change. Fitting Excel's cdf at a large n is T-3961's (p below 0.01) and T-4774's (Phase 1b).
- Sidecar values for the case's other not-converged Range!B cells, unless the same change gives them for free.
