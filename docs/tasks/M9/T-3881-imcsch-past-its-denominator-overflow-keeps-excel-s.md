---
id: T-3881
title: "IMCSCH past its denominator overflow keeps Excel's tiny parts and their sign (41 im-trig-magnitude-grid cells and argument-cut J16, ours 0)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Where the complex sinh in IMCSCH's reciprocal overflows, IMCSCH returns Excel's tiny result text, not 0. That covers
both parts, the odd sign rule on the real axis and the subnormal parts that print 0. The rule is stated once for the
whole overflow band, not as a list of points.

## Context
- Gap triage 2026-11-03 (`make eval` on main 08cba82c, excel-graph). `scripts/exit_owners.py --milestone M9` lists
  probes/im-trig-argument-cut J16 as the only M9 exit cell no open task owns. It belongs to the same group as the 41
  IMCSCH zeros of probes/im-trig-magnitude-grid. T-2668 (research, sonnet) and T-2914 (research, sonnet) cite those 41
  cells as probe questions. The grid answers T-2668's IMCSCH question (docs/excel-semantics/engineering.md, "IM* trig
  and hyperbolic functions past SIN's argument limit (T-3642)", the "Rule (not implemented, T-2668 and T-2914)"
  paragraph). No fix task exists, so this task owns the fix. T-2668 keeps its other question (whether other IM*
  functions flush a subnormal part).
- Excel vs ours (`make eval-show CASE=probes/im-trig-magnitude-grid.case.json`):
  - J134 `IMCSCH(COMPLEX(700,0))`: Excel 1.97193530875195E-304, ours 0. J135 `(700,1E-308)`: the same positive value
    (1E-308 is subnormal, so the input is probably real).
  - J136 `(700,1E-300)`: Excel -1.97193530875195E-304 (negative). J137 and J138 (1E-200, 1E-20) are the same.
  - J139 `(700,1)`: -1.06544119434148E-304+1.65932634623297E-304i. J141 `(700,700)`:
    1.65465944793074E-304+1.07267468194052E-304i.
  - J180 `(709,705)`: 2.33364281104964E-308i. J186 `(709,715)`: -2.3335595753309E-308i. The real part is
    subnormal there, so it prints nothing.
  - argument-cut J16 `IMCSCH(COMPLEX(700,134217727))`: Excel 5.78542952153839E-305-1.88515699993814E-304i, ours 0.
- Pattern checked by hand on these cells: a non-real z = x+yi at x = 700, 705 and 709 gives
  `-(2e^-x)·(cos y, -sin y)`, the negation of the mathematical csch. A real z gives `+2e^-x`. A part below the
  smallest normal double prints 0 (J146 `(700,710)` has no imaginary part, since sin 710 ≈ 6E-5). This is a
  hypothesis from the grid, not a verified rule. Excel is right even where the sign looks wrong: match it, and do not
  "fix" it (directive 1). Where the sign comes from is open. One candidate is the sign of an overflowed intermediate
  in Excel's complex division. Find a model that explains the sign rather than branching on it.
- Unprobed, so this needs new evidence before a rule is stated:
  - The band from where `|sinh z|²` overflows (x above about 355) up to 700.
  - Negative x. The grid has none below |x| = 709.5, where everything is 0.
  - Whether y = 1E-308 counts as real because COMPLEX or the text parse flushes it.
  Use the excel-probe skill. Suggested case: x in {360, 400, 524.561, 600, 699, 700} and their negatives, against
  y in {0, 1E-308, 1E-300, 1, 5.3113, 20, 705} and their negatives. Include the IMSECH companions, which T-2335 made
  keep tiny parts as `(cos y, -sgn(x)·sin y)·2e^-|x|` (engineering.md "Tiny results").
- 15-digit text: y's sine and cosine go through `trig_sin_cos` (Excel's x87 reduction, T-3758; J16's y = 2^27-1
  needs it). `2e^-|x|` is the scale `overflowed_reciprocal` already uses for IMSEC, IMCSC and IMSECH. Reuse both
  (directive 3).
- Code: `imcsch`, `reciprocal_result` and `overflowed_reciprocal` in
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.
- Cells owned:
  - probes/im-trig-magnitude-grid.case.json (41): Calc!J134:J149, J153:J168, J172:J176, J180, J183, J184, J186
  - probes/im-trig-argument-cut.case.json (1): Calc!J16
  - probes/t3881-imcsch-overflow-band.case.json (202): Calc!A3:A7, A9:A104, A107:A111, A113:A208 (every IMCSCH cell but
    the six real ones at x = ±355, which pass). The 12 failing IMSECH cells of column B are T-3883's.
  - probes/t3881-imcsch-band-edge.case.json (22): Calc!A3, A5, A7, A9, A11, A13, A15, A17:A20, A23, A25, A27, A29, A31,
    A33, A35, A37:A40
  - probes/t3881-imcsch-flip-start.case.json (20): Calc!A9:A28
  - probes/t3881-imcsch-flip-start-negative.case.json (12): Calc!A1:A12

## Acceptance criteria
- [ ] A probe case (excel-graph, canary-verified) covers the band and signs listed under Context. Its Excel values
      are recorded in docs/excel-semantics/engineering.md, and the IMCSCH rule there moves from "not implemented" to
      a stated rule with its evidence.
- [ ] The 42 owned cells match Excel's text, and the new probe case passes.
- [ ] The 42 known-failure lines (`F	probes/im-trig-magnitude-grid.case.json	Calc!J134` … J186 and
      `F	probes/im-trig-argument-cut.case.json	Calc!J16`) disappear from eval/baseline/probes.txt.
      `make eval-baseline` records no new known-failure line in any eval/baseline/<suite>.txt. The grid's other
      IMCSCH cells (the 0 and `#NUM!` cells from x = 709.5 up) keep passing.
- [ ] If choosing the 15-digit text means ranking kernel candidates (for example `2·exp(-x)` against
      `exp(-x)·2`, or 1/sinh through `inverse_exp_sinh_cosh`), the R4 guards (reviewer-excel-fidelity check 7,
      decision 0188) apply and the Outcome shows them:
  - [ ] Hold-out (R4.1): train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for
        the chosen candidate and the correctly rounded one, from the fit lab's split (T-3360). The margin is stated
        before scoring. The chosen candidate beats the correctly rounded one on hold-out.
  - [ ] Fresh sweep (R4.2): after the choice is frozen, a new probes-numeric case over the overflow band is
        oracle-filled and its counts are recorded.
  - [ ] Gate A envelope (R2): the fresh sweep's count inside the envelope (within 15 digits of Excel, or within
        1e-9 relative of 50-digit mpmath, absolute floor 1e-300), with every point outside listed.
  - [ ] `FITTED` tag (R4.3): any constant or cut chosen by scoring carries `FITTED(T-3881, n_cells, range)` and
        applies only inside that range. For a structural choice (kind (c)), the Outcome says so instead.
  - [ ] Reproducibility floor (R4.4): no step targets differences below the oracle's floor (decision 0112, web
        against desktop up to 4 ulp).
  - [ ] No keep-matched branch: no band or special case whose domain is defined by the cells that already match
        (decisions 0056, 0082).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decisions 0083 and 0114 (`semantics`): done when the 42 owned text cells equal Excel's text: the right parts, the
right sign and no part where Excel prints none. A leftover is recorded in engineering.md as UNVERIFIED with the
candidates tried. Bit-level residue in the 15th digit moves to a follow-up with `defer: phase-1b`.

## Out of scope
- The `#NUM!` cut at |y| = 2^27 and the zeros past x ≈ 709.09 (T-3642, done).
- IMCSCH of a large real number past `inverse_exp_sinh_cosh`'s band: T-2914, unless the new probe settles it.
  If it does, say so in T-2914.
- The 705+705i 15th-digit cells of the other reciprocals (T-3851). The subnormal flush of other IM* functions (T-2668).

## Progress (first run, 2026-11-03; partial PR)
The probe of the first acceptance item is done: four cases, filled from excel-graph and recorded in engineering.md
(the "Rule (not implemented, T-3881 ...)" paragraph). What they show:
- Inside the band, a non-real z gives -csch z, which equals -sgn(x)·IMSECH(z) digit for digit on every cell. A real
  z, or a subnormal y, gives the true sgn(x)·2e^-|x|.
- The band starts just past x = 354.5 for x > 0 and in (354, 354.01] for x < 0. Both edges sit below cosh 2x's
  overflow (354.8914) and sinh²x's (355.6). The grid's x = 700-709 cells are the same rule.
- The 12 IMSECH cells one digit off in this band are T-3883's. Reusing IMSECH's `overflowed_reciprocal` for IMCSCH
  would inherit those digits; Excel's IMCSCH texts are the exact negation of its IMSECH texts.
The acceptance criteria's "42 owned cells" and "42 known-failure lines" now cover these four cases too: 298 cells in
all (42 + 202 + 22 + 20 + 12).
Next step: pin the negative edge more finely (probe -354.001 to -354.009), look for the mechanism behind 354.5 and
354 (perhaps a test on exp(2x) or exp(-2x) against a cut), then implement in `imcsch` and drop the new cases'
known-failure lines.
