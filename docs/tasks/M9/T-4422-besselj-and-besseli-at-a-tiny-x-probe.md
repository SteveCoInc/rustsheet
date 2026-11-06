---
id: T-4422
title: "BESSELJ and BESSELI at a tiny x: probe where each order 2 to 7 turns #NUM! and match Excel's cut (the engine's cuts and its 0 results are unevidenced)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
For orders 2 to 7 and a tiny |x|, BESSELJ and BESSELI give what Excel gives: a number, 0 or `#NUM!`. Each order's
cut is bracketed by excel-graph cells, and the engine reproduces those brackets from the same mechanism Excel uses
(most likely its Miller recurrence overflowing at a huge 2/x), not from one threshold per order.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 20b163db, excel-graph sidecars). `make unowned-off ALL=1` gives 0
  unowned visibly-off cells, so this triage read the rules the semantics docs mark unprobed. The functions-engineering
  lane had no ready task.
- engineering.md "BESSELJ and BESSELI at a tiny x and a small order (T-4083)" gives the evidence: Gate A tables
  eval/accuracy/tables/BESSELJ/s1 and BESSELI/s1 (excel-graph, 2026-11-04). Excel answers
  `BESSELJ(1.16668964461302E-30,2)` (1.7014559085590685E-61), `BESSELJ(5.57080663109133E-14,4)` and
  `BESSELI(-1.02514099090909E-10,4)`. It is `#NUM!` at `BESSELJ(1.10541629485784E-181,2)`,
  `BESSELJ(2.49185320362869E-170,3)`, `BESSELJ(-6.6840116183702E-247,3)`, `BESSELJ(-3.63601369770004E-65,4)`,
  `BESSELJ(1.90005396436845E-196,4)` and `BESSELI(-3.53342392064674E-38,4)`. The entry says: "where each cut lies is
  unprobed". The engine matches all of those cells, so no committed cell fails today.
- The engine's own cuts were read from a scratch case on a release build of 20b163db, with x = 1E-4, 1E-6, ... 1E-306:
  - BESSELJ answers down to 1E-38 at order 2 (`#NUM!` from 1E-40), 1E-32 at order 3, 1E-26 at order 4, 1E-24 at order 5
    and 1E-22 at orders 6 and 7.
  - BESSELI answers down to 1E-22 at order 2, then gives **0.0** at 1E-24 and `#NUM!` from 1E-26. At order 4 it gives
    0.0 at 1E-18 and `#NUM!` from 1E-20. At order 6 it gives 0.0 at 1E-16 and `#NUM!` from 1E-18. Orders 3, 5 and 7
    go from a number straight to `#NUM!` (1E-20 / 1E-22, 1E-16 / 1E-18, 1E-14 / 1E-16).
  - The true values there are normal doubles: I_4(1E-18) is about 2.6E-75 and I_2(1E-24) about 1.25E-49. So the engine's
    0 results have no Excel evidence behind them, and nor do the engine's cuts in the gaps between Excel's answering and
    erroring cells (1E-30 to 1E-181 at J order 2; 1E-10 to 3.5E-38 at I order 4). Any of them may be visibly off.
- Code: `jn_miller` and `miller_stays_finite` in crates/rustsheet-engine/src/functions/engineering/transcendental/
  bessel_excel.rs, and `in_miller` and `bessel_i` in bessel_i.rs. The recurrence runs in emulated x87 steps
  (`extended_product`, `extended_sum`; T-2548). The rescale by `MILLER_RESCALE_FACTOR` can underflow `result` to 0.
  That is a candidate source of the 0 results.
- Design question, hence opus: is Excel's `#NUM!` the recurrence's own overflow (inf, then NaN), a pre-check like
  `miller_stays_finite`, or a result-underflow test? The answer decides whether the engine changes its recurrence or a
  guard. If the cut has to be chosen by scoring cells, it is a fit (reviewer-excel-fidelity check 7).

## Probe
excel-probe skill, excel-graph, tag M9. Name the case probes/t4422-bessel-tiny-x-cuts.
- Rows: x = 10^-k for k = 4, 6, ... 306 (the scratch grid), plus a bisection to 3 significant digits around each
  order's Excel cut once the grid has bracketed it. Use a second fill for this.
- Columns: BESSELJ and BESSELI, orders 2 to 7. Add a negative x for the odd orders, because the sign rule is
  engineering.md's.
- Wrap each call in `ERROR.TYPE` and `ISNUMBER`, so that a returned 0 can be told from a flushed subnormal (decision
  0042).

## Acceptance criteria
- [x] probes/t4422-bessel-tiny-x-cuts is committed with excel-graph oracle values. Every order 2 to 7 of both functions
      has its cut bracketed by two adjacent cells, one answering and one `#NUM!`, or it is shown never to error above
      1E-306.
- [x] Every cell of the case passes. The engine change is a model of Excel's mechanism, stated in the PR, and it also
      keeps the six #NUM! cells and three answering cells of eval/accuracy/tables/BESSELJ/s1 and BESSELI/s1 listed
      above. If no mechanism reproduces every bracket, the cells left over get a functions-engineering follow-up made
      with `make new-task`. That task names the cells and carries `campaign: semantics` and a `## Visible-parity stage`
      section.
- [x] If any cut or constant is chosen by scoring candidates against oracle cells (check 7 kind (a) or (c)), the
      Outcome shows these decision 0188 R4 guards:
      - hold-out: train and hold-out counts side by side, about 30% of the cells by cell hash, never tuned on;
      - fresh sweep: a new log-uniform probes-numeric case oracle-filled after the freeze;
      - the Gate A envelope result on that sweep, every point outside it listed;
      - a `FITTED(T-4422, n_cells, range)` tag, applied only inside `range`;
      - the reproducibility floor respected (no target below 4 ulp web against desktop);
      - no keep-matched branch.
- [x] engineering.md "BESSELJ and BESSELI at a tiny x" replaces "where each cut lies is unprobed" with the bracketing
      cells, and says whether Excel ever returns 0 there.
- [x] `make eval-baseline` records no regression; `rustsheet-eval accuracy` passes; `make check` green; reviewers approve.

## Out of scope
- BESSELK and BESSELY at a tiny x, and orders above 7 (file a follow-up if the mechanism generalizes and cells differ).
- BESSELI at a large x past R3's 1e-4 (T-4196).

## Outcome
- Probes (excel-graph, 2026-11-06): probes/t4422-bessel-tiny-x-cuts (2,736 cells: x = 10^-k, k = 4 to 306 step 2,
  BESSELJ and BESSELI orders 2 to 7, -x for the odd orders) and probes/t4422-bessel-tiny-x-bisect (432 cells: mantissas
  9..1 over the two bracketing decades plus 3-digit steps around each cut). Every order of both functions, at +x and
  -x, has its cut bracketed by adjacent 3-digit cells (engineering.md table). Every x below a cut is `#NUM!` to 1E-306;
  Excel never returns 0 there.
- Mechanism: the cut is exactly the first x at which the order's Miller recurrence overflows, and any overflow is
  `#NUM!`. All 18 Excel brackets equal the engine's own first non-number x. BESSELJ already matched. BESSELI flushed an
  overflow after the step j = n to 0 (`i0 / inf`), a band up to two decades wide below each cut; `in_miller` now returns
  NaN when its last term is not finite. No cut is chosen by scoring, so it is not a fit.
- BESSELI's values: plain doubles missed 8 cells by 1 ulp (`BESSELI(±1E-6,7)` and six order-6 cells). The model is a
  structural choice from three ranked candidates (decision 0188 R4, check 7 kind (b)): double rounding at every step and
  rescale (fixed 3 cells), then also a double-rounded `ans * (i0/bi)` (fixed all 8). An 80-bit `i0/bi` held in a register
  missed 58. Hold-out: no split was made before scoring. The candidates were scored on both whole cases, so the
  fresh sweep below is the only held-out evidence.
  - Fresh sweep, built after the model was frozen: probes-numeric/t4422-besseli-fresh-sweep (400 cells,
    log-uniform |x| in [1E-15, 700], a third negative, orders 2 to 20, seed 4422, excel-graph). It passes 400/400
    bit for bit.
  - Gate A (`rustsheet-eval accuracy`): 0 failing, BESSELI/s1 included. No `FITTED` tag: no constant was fitted.
  - Reproducibility floor: not engaged, since every target is bit-exact. Keep-matched branch: none.
- engineering.md "BESSELJ and BESSELI at a tiny x and a small order" now carries the brackets and the rounding rule. The
  "undetermined" note on BESSELI's double rounding in "BESSEL order cap" is resolved.
- Eval delta: +3,568 cells, all passing (3 new cases). No other cell changed: 0 fixed, 0 regressions. The engine's 0 cells
  that used to fall in the gaps between Excel's cells are now `#NUM!`, as Excel gives.
- Follow-ups: none. Every cell passes.
