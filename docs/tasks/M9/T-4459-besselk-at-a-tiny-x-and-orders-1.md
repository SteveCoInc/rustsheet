---
id: T-4459
title: "BESSELK at a tiny x and orders 1 to 7: probe where Excel turns #NUM! (K1's 1/x term or the upward recurrence overflowing) and match the cut"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
For BESSELK at a tiny positive x and orders 0 to 7, the engine is `#NUM!` exactly where Excel is and gives Excel's
number everywhere else, with each order's cut bracketed by adjacent Excel cells. T-4422 did the same for BESSELJ and
BESSELI. This is BESSELK's half of the question. BESSELY's half is T-4458.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 16a00ebc7). There is no unowned visibly-off cell. This task
  covers an evidence gap next to T-4422's finding (engineering.md "BESSELJ and BESSELI at a tiny x and a small
  order": each cut is the first x where the order's recurrence overflows, and Excel never returns 0 below it).
- BESSELK's tiny-x Excel cells are all far from any cut:
  - `BESSELK(1E-300,3)` is `#NUM!` (gen-m9/060b18586fcff6e6 Calc!A214);
  - `BESSELK(1E-10,0)` is 23.1417824505004 (gen-m9/3cf48801617a72ad Calc!A88);
  - `BESSELK(1E-300,100001)` is past the order cap (probes/t2546-bessel-past-cap C15).
- The engine (crates/rustsheet-engine/src/functions/engineering/transcendental/bessel_k.rs) follows NR's `bessk`.
  - K0 at x <= 2 is `-ln(x/2) I0(x) + poly`. It stays finite down to the smallest subnormal (about 744), so order 0
    should never be `#NUM!` for x > 0. The probe checks this.
  - K1 at x <= 2 is `ln(x/2) I1(x) + (1/x) poly`. Its `1/x` overflows below 1/DBL_MAX, about 5.56E-309, where the
    true K1 (about 1/x) overflows too. The cut may still sit an ulp of x away, depending on whether Excel keeps
    `1/x` in an x87 register.
  - Orders 2 and up use the upward recurrence `bkm + flush_subnormal(j * tox * bk)` with `tox = 2/x`, in plain
    doubles. K_n(x) is about (n-1)!/2 (2/x)^n, so order 2 overflows near x = 1.05E-154 and order 3 near 2.6E-103.
    Two things are unprobed: whether the product `(j * tox) * bk` overflows before the sum is stored, and whether
    `tox` itself overflows at x below 1.1E-308.
- Probe design (T-4422's layout, probes/t4422-bessel-tiny-x-cuts and t4422-bessel-tiny-x-bisect):
  - For orders 0 to 7, decades 10^-k around each order's predicted overflow, then mantissas 9..1 over the two
    bracketing decades, then 3-digit steps around the cut.
  - For order 1, cells at 1/DBL_MAX and its neighbours.
  - Put the new case under eval/suites/probes (decision 0267 item 5 freezes probes-numeric).
- Not a fit if the cut is an overflow rule (kind (b) of reviewer-excel-fidelity check 7: x87/IEEE semantics). A
  rounding model for the digits next to the cut, chosen by scoring candidates, is kind (c) and owes the guards below.
  BESSELI's recurrence rounds twice (T-4422), while BESSELK's rounds once. Whether K's double rounding matters near the
  cut is part of that question.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes, oracle-filled from excel-graph, brackets the `#NUM!` cut of each order 1
      to 7 between adjacent cells, three significant digits or finer (order 1 across 1/DBL_MAX). It also records
      order 0 down to 5E-324.
- [ ] engineering.md records the brackets and the mechanism, citing the cells, next to "BESSELJ and BESSELI at a tiny
      x and a small order" (or in it).
- [ ] Every cell of the new case passes. No known-failure line is added to any eval/baseline file, and none of the
      existing BESSELK cells (probes/eng-besselk-underflow-bit-shift, probes-numeric/m9-besselk-underflow,
      probes/t2546-bessel-past-cap, ironcalc-calc BESSEL.xlsx, gen-m9) changes verdict.
- [ ] If the digits next to the cut need a rounding model chosen by scoring (kind (c) of reviewer-excel-fidelity
      check 7, decision 0188 R4), the Outcome shows:
      - the hold-out: train and hold-out counts side by side for the chosen and the correctly rounded candidate, with
        the split (about 30% by cell hash) and its margin stated before scoring;
      - a fresh log-uniform sweep, under eval/suites/probes, oracle-filled after the model froze;
      - the Gate A envelope result on that sweep, with every point outside listed;
      - no `FITTED` tag unless a constant is fitted (kind (a)); if one is, a `FITTED(T-4459, n_cells, range)` tag and
        accurate math outside the range;
      - no target below the reproducibility floor (4 ulp web against desktop, bits that vary between fills);
      - no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BESSELY (T-4458), BESSELK's subnormal recurrence-term flush at a large x (T-2702), orders past the cap.
