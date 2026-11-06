---
id: T-4112
title: "Gate A tables, discount securities and T-bills: DISC, INTRATE, RECEIVED, PRICEDISC, YIELDDISC, TBILLEQ and TBILLPRICE"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
DISC, INTRATE, RECEIVED, PRICEDISC, YIELDDISC, TBILLEQ and TBILLPRICE each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0273 (T-3797) classified these functions as needing a table.
- Date arguments are integer serials. An order constraint (settlement < maturity, issue < settlement < first coupon) is a
  Definition's joint `draw` or a `domain` rule (`accuracy/model.py`), as T-3535's NPER and RATE use (`defs/annuity.py`).
  Basis is an `int` parameter over 0..4. Reuse `defs/yearfrac.py` for the day counts (directive 3).
- Solvers follow the IRR and RATE pattern (T-3535, T-3586): `solver=True`, 300 points, and Excel's root choice cited from
  a semantics entry. A point with several roots is not drawn without one.
- Where Excel's own arithmetic (a rounding step, an operation order) puts it far from the math, list the points in the
  Outcome as R3 candidates for an owner ruling and do not mark them (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Sources (financial.md): "Discount-security functions: settlement 0, basis and argument modes", "Discount securities:
  day count and year length", "TBILLEQ and TBILLYIELD".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve. (`make check-fast` green locally; review pending)

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/discount.py`: one definition each for DISC, INTRATE, RECEIVED, PRICEDISC,
  YIELDDISC, TBILLEQ and TBILLPRICE, the exact quotients on the double arguments. The five securities read the basis
  day count D and year length B apart through `day_count_parts`, factored out of defs/yearfrac.py together with
  `whole_serial`, `check_basis` and `serial_of` (YEARFRAC now uses them; directive 3). The T-bills count serial days,
  and their one-year limit reads the displayed dates (29 February's anniversary is 1 March, 1900-01-00's 1900-12-31).
  TBILLEQ from 183 days is the root of the semi-annual equivalence. No solver.
  - Domain errors, each from a semantics entry or an eval cell, all `#NUM!`: dates off the calendar, settlement on or
    after maturity, basis outside 0..4; prices, investments and redemptions at most 0, PRICEDISC's and RECEIVED's
    discounts at most 0; RECEIVED where 1 - r·D/B is at most 0 and at settlement 0; the T-bills' more-than-a-year
    maturity, a 9999 settlement, a negative discount and a negative price; TBILLEQ's zero price from 183 days.
  - No math value where nothing evidences Excel's result (DISC at redemption 0, a T-bill discount of 0, a 30/360 day
    count of 0) or where the value cancels (`ill_conditioned`). That includes a price within cancellation of 0:
    `TBILLEQ(1000,1100,3.6)` is Excel's 0 only because 3.6·100 rounds to 360. Before generating, the definitions were
    run over all 945 literal one-call cells (with an oracle value) of these functions in eval/suites. No math domain error falls where
    Excel has a number, and every math value is within 1e-12 of Excel. The first pass coded DISC at settlement 0 as
    no value and was corrected from probes/discount-serial-zero.
  - Tests in tools/tests/test_accuracy_discount.py: Microsoft's examples, the semantics entries' Excel values, the
    evidenced domains, the unprobed points and the draws.
- Tables `eval/accuracy/tables/{DISC,INTRATE,RECEIVED,PRICEDISC,YIELDDISC,TBILLEQ,TBILLPRICE}/s1`, 200 points each
  (1,400), filled from excel-graph with 0 failures. Every error point in Excel matches a coded domain, and each is `#NUM!`.
- `rustsheet-eval accuracy`: all 1,400 points inside the envelope (1,388 on both sides, 11 TBILLEQ points through
  Excel's 15 digits only, 1 through the math only), 0 failing, 0 known failures added. `missing tables` went from
  121 to 114. The engine is bit-exact with Excel on 1,398 of 1,400. The other two are TBILLEQ from 183 days, a few ulps
  apart and both near the math (A39 `TBILLEQ(33842,34067,0.213177283226975)`, A127 `TBILLEQ(45351,45716,0.178249140751279)`).
- **Excel kept as the target (decision 0300):** no point is more than 1e-4 relative off the math. The largest are
  TBILLEQ from 183 days at tiny discounts (A110 8.7E-5, A175 5.7E-5), where Excel's `-t + sqrt(...)` cancels. The engine
  is bit-exact there.
- New quirks and domain rules, in financial.md "Discount securities and T-bills: amount domains (T-4112)": DISC's
  price and negative-redemption domain, RECEIVED's zero and negative discount and its 1 - r·D/B below 0 (until now
  only inferred), PRICEDISC's negative discount and overflow, the T-bills' negative discount, DISC at settlement 0, the
  zero-price rounding, and TBILLEQ's cancellation at tiny discounts.
- Math sidecars: `rustsheet-accuracy math --function` over the seven functions, for probes (probes-numeric has no cell
  of them), run after merging origin/main. That gives 10 new sidecars and 98 cells added to 8 existing ones, with no
  existing cell changed. `make eval-math-check` prints "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
