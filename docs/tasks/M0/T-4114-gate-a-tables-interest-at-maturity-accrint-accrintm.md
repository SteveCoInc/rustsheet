---
id: T-4114
title: "Gate A tables, interest at maturity: ACCRINT, ACCRINTM, PRICEMAT, YIELDMAT and TBILLYIELD"
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
ACCRINT, ACCRINTM, PRICEMAT, YIELDMAT and TBILLYIELD each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Sources (financial.md): "ACCRINT at the calendar's start", "ACCRINT with calc_method TRUE: a next coupon past the
  calendar is #NUM!", "Discount-security functions: settlement 0, basis and argument modes", "TBILLEQ and TBILLYIELD".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0273.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/accrint.py` (ACCRINT) and `defs/discount.py` (TBILLYIELD beside the T-bills,
  PRICEMAT, YIELDMAT and ACCRINTM beside the discount securities): one definition each, exact on the double
  arguments. They reuse defs/yearfrac.py (`day_count_parts`, `whole_serial`, `check_basis`, the 30/360 counts, with a
  new `us_30_360_both`) and fitlab's month stepping (`add_months`, `month_end`, made public in
  fitlab/kernels/solvers.py). The T-bills' date checks are factored into `_tbill_days`. No solver.
  - ACCRINT ports the engine's `compute_accrint` schedule (first_interest's chained schedule, calc_method, the issue's
    stub), with one change: Excel counts the issue's period on basis 2 as on basis 0, US 30/360 moving both dates
    (probes/accrint-zero-results C1:C26, accuracy/ACCRINT/s1 A22). Serials below 61 (Excel's 1900-01-00 to 02-29) are
    not modelled and have no math value.
  - Domain errors, all `#NUM!` and each evidenced: dates off the calendar (ACCRINT's settlement past 9999 included),
    reversed dates, an issue on or after settlement (PRICEMAT, YIELDMAT, ACCRINT), bases outside 0..4 (ACCRINT: negative
    only), rates at most 0 and negative pars (ACCRINT, ACCRINTM), negative rates and yields (PRICEMAT), negative rates
    and prices (YIELDMAT), the T-bills' date rules (TBILLYIELD), ACCRINT's coupon dates before 1900-01-01 (calc_method
    FALSE or settlement before first_interest) and its next coupon past 9999 (bases 0 and 1).
  - Math values from evidence: ACCRINTM with issue equal to settlement is 0 (probes/accrint-zero-results B1:B40);
    PRICEMAT's yld of 0 computes (gen-m9 9a1fa5366bc50918 A180).
  - No math value (unprobed; added to T-4522): TBILLYIELD at a price at most 0, YIELDMAT at a price of 0, a par of 0,
    ACCRINT's frequency other than 1, 2, 4 and basis from 5, a 30/360 DSM at most 0 (PRICEMAT, YIELDMAT), values past
    the double range, cancellation (`ill_conditioned`), and ACCRINT with calc_method TRUE, settlement after
    first_interest and a coupon date before 1900-01-01, where Excel for the web gave 0 on 2026-11-06 and `#NUM!` in
    earlier fills.
  - Before generating, the definitions were run over all 991 literal one-call cells of the five functions in
    eval/suites that fit them: 452 numbers, 358 errors, 181 without a math value. No math domain error falls where
    Excel has a number, no math value where Excel has an error, and every number is within 1e-12 of Excel. The first
    pass coded negative rates, pars and yields as unprobed; those cells (gen-m9 and probes) evidence `#NUM!`, and PRICEMAT's
    yld of 0 a number, so the definitions follow them.
  - Tests in tools/tests/test_accuracy_maturity.py (Microsoft's examples, the entries' and eval cells' Excel values,
    the domains, the unprobed points, the draws).
- Tables `eval/accuracy/tables/{ACCRINT,ACCRINTM,PRICEMAT,YIELDMAT,TBILLYIELD}/s1`, 200 points each (1,000), filled
  from excel-graph with 0 failures. The first fill showed four table defects (Excel 0 where the definitions said
  `#NUM!`): ACCRINTM with equal dates (A115, A134) and ACCRINT at the calendar's start (A127, A131 of that fill). The
  probe probes/accrint-zero-results (85 cells) settled both; ACCRINT's table was regenerated and refilled.
- `rustsheet-eval accuracy`: 0 failing, 0 table defects. `missing tables` lower by 5, to 60 after
  the final merge of origin/main (78 at the first run, before merges brought other groups' tables). The engine is bit-exact with Excel on 996 of 1,000 points; PRICEMAT A101 is 1 ulp off, inside
  the envelope. Three points are known failures under T-4566 (functions-financial, filed here): ACCRINTM A115 and A134
  (equal dates: Excel 0, the engine `#NUM!`) and ACCRINT A22 (the engine's basis-2 issue period, 9E-4 off Excel).
- **Excel kept as the target (decision 0300):** no table point and no probe sidecar cell of these functions is more
  than 1e-4 relative off the math. The largest table difference is ACCRINT A11
  `ACCRINT(49116,49633,49117,0.00183790807306151,21.6508879408564,2,2)`, -2.2E-18 in Excel where the math is 0 (a
  regular-period slice and a whole period cancel), inside through Excel's 15 digits.
- Semantics: new financial.md entry "ACCRINT, ACCRINTM, PRICEMAT and YIELDMAT: amount domains (T-4114)" (the amount
  domains, ACCRINTM's 0 at equal dates, ACCRINT's settlement past 9999, its basis-2 issue period and its 0 at the
  calendar's start); "ACCRINT at the calendar's start" amended with the 2026-11-06 zeros.
- Math sidecars: `rustsheet-accuracy math --function` over the five functions for probes (probes-numeric has none of
  their cells), re-run after the final merge: 13 new sidecars and cells added to 18 existing ones, no existing cell
  changed. `make eval-math-check` prints "current".
- Follow-ups: T-4566 (engine: ACCRINTM equal dates, ACCRINT basis-2 issue period, a re-fill of the calendar-start
  probes); T-4522 extended with this task's unprobed points.
- Eval delta: the new probe probes/accrint-zero-results adds 57 visibly-off cells (ACCRINTM's equal dates, ACCRINT's
  zeros and basis-2 cells), recorded by `make eval-baseline` and owned by T-4566; no other gated cell changed.
