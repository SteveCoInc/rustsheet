---
id: T-3367
title: "NPER at a positive rate below 1E-14 follows the math, not Excel's up-to-4% error (decision 0188 R3)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3355]
---

## Goal
NPER at 0 < rate < 1e-14 (excluding rates where 1 + rate == 1,
which stay `#DIV/0!`) returns the math (the zero-rate limit and its first-order correction) instead of reproducing
Excel's result, which is up to 4% off there (decision 0188 R3, `MATH-OVER-EXCEL`).

## Context
- Owner's 2026-10-31 assessment (decision 0188, "Where Excel itself is far from the math"): NPER at rates below 1e-14 is up
  to 4% off in Excel, above G = 1e-4, so the engine follows the math there; financial.md's `MATH-OVER-EXCEL` entry
  records it. Today the engine matches Excel bit for bit.
- Code: `fn_nper` in crates/rustsheet-engine/src/functions/financial/annuity.rs.

- Gate A (T-3535, decision 0220) R3-marks 12 points of accuracy/NPER/s1 inside the region: A121, A130, A133, A135,
  A142, A143, A151, A152, A153, A156, A158 and A162, at rates 3.0E-16 to 7.6E-15, with Excel up to 35% off. They are
  `known-failures.tsv` lines owned by this task; remove them once the engine returns the math there.

## Acceptance criteria
- [ ] A probe case (one milestone tag, M9), filled by the oracle, holds NPER points at rates from 1e-17 to 1e-12 with their
      50-digit math values; the financial.md entry cites it and states the region where Excel's error exceeds 1e-4.
- [ ] In that region the engine is within 1e-9 relative of the math; outside it the engine still matches Excel (within
      15 digits).
- [ ] Every cell where Excel returns an error (`#DIV/0!` where 1 + rate == 1, `#NUM!`) keeps Excel's error.
- [ ] Negative tiny rates are out of the region; if they are to be covered, probe them first (a separate acceptance in
      a follow-up task).
- [ ] Cells that leave Excel's value are listed in the Outcome; no other gated cell regresses.
- [ ] Before any engine change follows the math, the region's evidence points (oracle-filled, with 50-digit math
      values) and its bounds are committed and the `MATH-OVER-EXCEL` entry cites them; the engine change comes after,
      in the same PR or a later one.
- [ ] The math path keeps directive 5's caps (bounded iterations, no panic); decision 0188 R3 never overrides them.
- [ ] R3 cells are marked as the Gate A spec (T-3355) says. Interim route until T-3359 lands: the cells that leave
      Excel's value are listed in the Outcome, they still count as visibly off in the report, and their baseline
      regression is recorded with a decision record citing decision 0188 R3.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other financial functions.

## Outcome
- New probe case probes-numeric/t3367-nper-tiny-rate, 140 cells: NPER at 20 positive rates from 1E-17 to 1E-12 with
  seven argument sets each, all five arguments written. It is excel-graph filled (2026-11-05) with a 50-digit math
  sidecar (`rustsheet-accuracy` NPER definition, written for this case only).
- Excel against the math: inside the region (1.2E-16 to 9.9E-15) it is 6.5E-3 to 4.7E-1 off, up to 46% at 1.2E-16. At
  1E-14 to 1E-13 it is still 7.0E-4 to 6.9E-3 off, and at 3E-13 and 1E-12 it is under G (5.8E-5 to 9.1E-5).
  financial.md's MATH-OVER-EXCEL entry states this region and cites the probe.
- Engine (`nper_of`, functions/financial/annuity.rs): for 0 < rate < `NPER_MATH_RATE` (1E-14), once Excel's error checks
  pass, the value is log1p(-(fv+pv)·rate/(pmt·k+pv·rate)) / log1p(rate). That is within 3.4E-16 of the math on all 66
  numeric cells in the region. Outside it every cell keeps Excel's value: all 74 other cells match Excel exactly.
- Order fix with new evidence: the 1 + rate = 1 `#DIV/0!` now comes before a non-positive ratio's `#NUM!`.
  `NPER(1E-17,0,100,200,0)` is `#DIV/0!` in Excel (A7, A14, A21). financial.md had inferred the reverse order without a
  probe; it now records the probed order, and T-2749's context points at it.
- Cells that leave Excel's value: the 66 numeric cells A22 to A97 of the probe. They are R3-marked in
  eval/accuracy/math-over-excel.tsv (decision 0282), and the 12 Gate A points of decision 0220 are already marked. No
  other eval cell changed. `make eval-baseline` reports 0 fixed and 0 regressions in every gated suite, and
  probes-numeric is not baselined.
- Gate A: the 12 T-3367 known-failures lines are removed (accuracy/NPER/s1 A121 to A162). The points now pass on the
  math side. Gate A shows 0 failing and 0 table defects.
- R3 candidates outside the ruled region: the 24 cells at 1E-14 to 1E-13 (A99 to A125). They are listed for an owner
  ruling in T-4285 (blocked: needs the owner), not marked.
- Directive 5: the math path is two `ln_1p` calls and a division, with no loop and no new allocation.
- Unit tests: `nper_below_a_rate_of_1e_14_is_the_math` and `nper_from_a_rate_of_1e_14_keeps_excels_value`
  (test_fn_annuity_fits.rs).
- Not a fit: the region bound is the owner's (decision 0188), and the formula is the exact identity.
- Eval delta: probes-numeric gains 1 case of 140 cells. 74 match Excel, and 66 are R3 math cells that pass R1(c) and
  the new-case owner gate. The gated suites show 0 fixed, 0 regressions and no baseline change.
- Follow-up: T-4285.
