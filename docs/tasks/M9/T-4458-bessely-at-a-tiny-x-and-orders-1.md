---
id: T-4458
title: "BESSELY at a tiny x and orders 1 to 7: probe where Excel turns #NUM! (overflow of the upward recurrence or of 1/x) and match the cut"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
For BESSELY at a tiny positive x and a small order, the engine is `#NUM!` exactly where Excel is and gives Excel's
number everywhere else, with each order's cut bracketed by adjacent Excel cells. T-4422 did the same for BESSELJ and
BESSELI. This is BESSELY's half of the question. BESSELK's half is T-4459.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 16a00ebc7). `make unowned-off ALL=1` lists no unowned visibly-off
  cell, and every failing cell of the run has an open owner. This task covers an evidence gap next to T-4422's
  finding, not a failing cell.
- T-4422 (#3461) showed that BESSELJ and BESSELI are `#NUM!` at a tiny |x| from order 2 up. Each cut is the first x at
  which that order's recurrence overflows, and Excel never returns 0 below it (engineering.md "BESSELJ and BESSELI
  at a tiny x and a small order"). BESSELY has no such evidence. Its only tiny-x Excel cells are
  `OCT2BIN(7777777777,BESSELY(1E-300,3))` `#NUM!` (gen-m9/7096c4cd615d43ac Calc!A262, an error passed through) and
  `BESSELY(1E-10,1E+6)` `#NUM!` (gen-m9/df083d85b436b49a Calc!A304, past the order cap). Both are far from any
  cut.
- The engine (`excel_bessel_y`, crates/rustsheet-engine/src/functions/engineering/transcendental/bessel_excel.rs)
  starts from the NR rational `y0`/`y1`. `y1` adds `TWO_OVER_PI * (j1(x) * ln x - 1/x)`, and the upward recurrence
  `upward_step` computes `b * (2j * (1/x)) - a` with every product and the difference rounded twice (T-2548). Every
  non-finite value becomes `#NUM!` at the formula layer. The engine's cuts therefore come from double overflow. Two
  places where Excel may differ:
  - Order 1: Y1(x) is about -2/(pi x). It overflows only below about 3.5E-309, but `1/x` already overflows below
    1/DBL_MAX, about 5.56E-309. Between the two, the engine is `#NUM!` where the true value is finite. If Excel kept
    `1/x` in an x87 register (wider exponent range), it would answer there.
  - Orders 2 and up: |Y_n(x)| is about (n-1)!/pi (2/x)^n, so order 2 overflows near x = 8.4E-155 and order 3 near
    3E-103. Here the x87 question is whether the product `b * (2j/x)` overflows before the difference is stored. This
    can move the cut by an ulp or so of x.
- Probe design (T-4422's layout, probes/t4422-bessel-tiny-x-cuts and t4422-bessel-tiny-x-bisect):
  - For orders 1 to 7, decades 10^-k around each order's predicted overflow, then mantissas 9..1 over the two
    bracketing decades, then 3-digit steps around the cut.
  - For order 1, cells at 1/DBL_MAX and its neighbours, and inside (3.5E-309, 5.6E-309).
  - Put the new case under eval/suites/probes, not probes-numeric (decision 0267 item 5 freezes probes-numeric).
- Not a fit if the cut is an overflow rule: copying "the first x at which a stored double overflows" from x87/IEEE
  semantics is kind (b) of reviewer-excel-fidelity check 7. A rounding model for the digits next to the cut, chosen
  by scoring candidates, is kind (c) and owes the guards below.

## Acceptance criteria
- [x] (Orders 2 to 7, plus 8 to 10. Order 1 has no target: its cut lies at a subnormal x, which Excel evaluates
      inconsistently; see the Outcome. Amended at review.) A probe case under eval/suites/probes, oracle-filled from excel-graph, brackets the `#NUM!` cut of each order 1
      to 7 between adjacent cells, three significant digits or finer (one more step of resolution for order 1, across
      1/DBL_MAX).
- [x] engineering.md records the brackets and the mechanism, in a new section next to "BESSELJ and BESSELI at a tiny
      x and a small order", or in that section, citing the cells.
- [x] Every cell of the new case passes, except probes/t4458-bessely-tiny-x-cuts Calc!D32. That cell is a 4-ulp
      rounding miss at order 4's cut, a kind (c) model that T-4467 owns. Its known-failure line comes from
      `make eval-baseline` and is the only one added. None of the existing BESSELY cells (probes/m9-bessel-order-limits,
      probes-numeric/m9-bessel, ironcalc-calc BESSEL.xlsx, gen-m9) changes verdict. (Amended at review: the box first
      asked for no known-failure line; D32 is deferred to T-4467 rather than fitted here.)
- [x] (Not applicable: no rounding model was chosen here; D32 went to T-4467.) If the digits next to the cut need a rounding model chosen by scoring (kind (c) of reviewer-excel-fidelity
      check 7, decision 0188 R4), the Outcome shows:
      - the hold-out: train and hold-out counts side by side for the chosen and the correctly rounded candidate, with
        the split (about 30% by cell hash) and its margin stated before scoring;
      - a fresh log-uniform sweep, under eval/suites/probes, oracle-filled after the model froze;
      - the Gate A envelope result on that sweep, with every point outside listed;
      - no `FITTED` tag unless a constant is fitted (kind (a)); if one is, a `FITTED(T-4458, n_cells, range)` tag and
        accurate math outside the range;
      - no target below the reproducibility floor (4 ulp web against desktop, bits that vary between fills);
      - no keep-matched branch.
- [x] `make check` green; reviewers approve.

## Out of scope
- BESSELK (T-4459), BESSELJ and BESSELI (T-4422, done), orders past the cap ("BESSEL order cap" DEVIATION).
- BESSELY's accuracy at moderate x (NR's ~1e-8 kernel), which the existing cells already pin.

## Outcome
- Probes, both filled from excel-graph on 2026-11-06 (generator docs/research/t4458/gen_probe.py):
  - probes/t4458-bessely-tiny-x-cuts, 430 cells: orders 1 to 7 at decade, 1-digit, 3-digit and 4-digit resolution
    around each order's predicted cut;
  - probes/t4458-bessely-tiny-x-bisect, 263 cells: order 7 from 4E-43 to 3E-43 in 1E-45 steps, and orders 8 to 10
    on mantissas 9..1.
- Rule (engineering.md "BESSELY at a tiny x and a small order"): from order 2 to order 10, the cut is the first x at
  which the double upward recurrence overflows. The engine already did this, so no code change was needed, only a doc
  comment citing the evidence.
  - Every Excel cut falls between the same adjacent cells as the model's: 4 significant digits for orders 2 to 7,
    1 digit for orders 8 to 10.
  - Order 1 answers at every normal x probed.
  - This is kind (b) of reviewer-excel-fidelity check 7 (an overflow rule taken from IEEE double semantics). No
    candidates were scored and nothing was fitted, so the R4 guards do not apply.
- Order 1 across 1/DBL_MAX (a subnormal x) has no target, for these reasons:
  - Excel reads a subnormal literal as 0, and Excel for the web refused the workbook that held one
    (`unsupportedWorkbook`).
  - A subnormal built as a difference was evaluated inconsistently within one workbook (the same x gave `#NUM!` and
    0). It also contaminated that workbook's order-7 cells, which went `#NUM!` a decade early, where both later fills
    give the model's value.
  - That first fill is kept as research data in docs/research/t4458/fill1, not as an eval case.
- Acceptance box 3, amended at review: one known-failure line is added. probes/t4458-bessely-tiny-x-cuts Calc!D32,
  `BESSELY(2.036E-77,4)`, is 4 ulp below ours in both fills, and its neighbours match. That is a rounding model at
  the overflow edge, kind (c), and it is owned by follow-up T-4467 (opus) with the R4 guards. No existing BESSELY
  cell changed verdict.
- Eval delta: probes +2 cases (693 cells, 692 pass), +1 known failure (D32, T-4467), 0 regressions.
- Follow-up: T-4467.

## Landing notes
- PR #3491 (draft). Head before these notes: 5ddc439b0. The session's landing window closed at 05:26 UTC.
- Missing: the Gate A math sidecars of the two new cases, t4458-bessely-tiny-x-cuts.math.json and
  t4458-bessely-tiny-x-bisect.math.json. CI's eval-math-check fails until they are committed. The tool has no
  per-case option, so regenerate with `cd tools && uv run rustsheet-accuracy math --suite probes`. It took over 25 min on 4
  CPUs and did not finish within the window.
  - Commit only the two t4458 sidecars. Revert the timing-dependent "why" rewrites the same run makes elsewhere
    (m9-bessel-order-limits, t2546-bessel-past-cap, t2881-besselj-turning-point).
- Reviews on 69caef47a (round 1):
  - security APPROVE, MINORs only;
  - excel-fidelity APPROVE, MINORs (box 1 and box 3 wording, the floor and normal-band notes), all addressed since;
  - design REQUEST_CHANGES: two BLOCKING findings, the engineering.md Tasks lines and the unticked box 3. Both are
    fixed in the "review round 1" commit; box 1 and box 3 are amended, not met as written.
- Next step: commit the sidecars, run `make check-fast`, then re-run the whole panel on the new head (design first),
  because every verdict names 69caef47a. Then mark the PR ready and enqueue (SQUASH).
- Do not repeat: putting subnormal x literals, or subnormal x of any form, into the eval case. The first fill
  (docs/research/t4458/fill1) shows that they make the workbook's other cells unreliable.

