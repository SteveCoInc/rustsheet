---
id: T-4425
title: "IMTAN and IMCOT past the 2^27 cut: probe the |y| where #NUM! turns into ±i (inferred from cosh 2y overflowing near 355.24, bracketed only by y = 20 and 700)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
For |x| at or above 2^27, IMTAN and IMCOT switch from `#NUM!` to `±i` at the same |y| as Excel. That switch is
bracketed by Excel cells to 1E-10, and the engine's switch is moved there if it differs.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 20b163db, excel-graph sidecars). `make unowned-off ALL=1` gives 0
  unowned visibly-off cells, so this triage read the rules the semantics docs mark unprobed.
- engineering.md, in the entry with the IM* 2^27 cut: "IMTAN and IMCOT stay `±0 ± i` once cosh 2y overflows
  (`IMTAN("1E+10+700i")` is i, `IMCOT("134217728+700i")` -i, `IMTAN("1E+10+20i")` `#NUM!`)". It then adds: "Inferred, not
  bracketed: the IMTAN/IMCOT exception starts where cosh 2y overflows (|y| about 355); the grid only shows y = 20
  (`#NUM!`) and y = 700 (±i) past the limit."
- Evidence: probes/im-trig-magnitude-grid (excel-graph, 2026-11-01):
  - `#NUM!` at Calc!C311 `IMTAN(COMPLEX(1E+10,20))`, F311 (IMCOT), and C330, F330, C349, F349 (x = 1E+100, 1E+300);
  - `i` / `-i` at C312, F312, C331, F331, C350, F350 (y = 700);
  - no cell between them.
- Ours: `imtan` and `tan_parts` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs return `#NUM!`
  while `past_trig_limit(x) && f64::cosh(2.0 * y).is_finite()`. `imcot` returns `∓i` once `sec_sinh_cosh(2.0 * y)`
  (libm's `cosh` outside the inverse-exp band) is infinite. So the engine switches where libm's cosh overflows,
  2y = 710.4758600739439 (y about 355.2379). A scratch case on a release build of 20b163db gives `IMTAN("1E+10+355i")`
  `#NUM!` and `IMTAN("1E+10+356i")` `i`.
- Excel's real COSH overflows sooner, where e^x does (2y = 709.7827, y about 354.8914; numbers.md "SINH and COSH
  overflow is `#NUM!` at once"). If Excel's IMTAN takes that cosh, the engine is `#NUM!` for y in
  (354.8914, 355.2379) where Excel is `±i`. T-4423 probes the real band. This task is the complex one and does not wait
  on it.

## Probe
excel-probe skill, excel-graph, tag M9. Name the case probes/t4425-imtan-imcot-switch.
- IMTAN and IMCOT of `COMPLEX(x, y)`:
  - x = 1E+10, 134217728 and -1E+10;
  - y = 354.8, 354.89, 354.8914, 354.892, 354.9, 355, 355.2, 355.237, 355.2379, 355.238, 355.3 and 356, plus the
    negative of each y.
- A second fill bisects to 1E-10 inside whichever interval switches.
- Controls: the same y with x = 1, where both models answer, to show the real part at the switch.

## Acceptance criteria
- [ ] probes/t4425-imtan-imcot-switch is committed with excel-graph oracle values, and the switch is bracketed by
      adjacent cells for each x.
- [ ] Every cell passes. If Excel's switch differs from the engine's, the `cosh 2y` finiteness test in `imtan` (and
      IMCOT's matching one) takes the cosh Excel evidently uses, with a unit test at each bracketing cell. If the
      bracket matches neither libm's cosh nor e^x overflow, hand the task up (work-loop step 5b) rather than fit a
      constant.
- [ ] engineering.md's "Inferred, not bracketed" sentence is replaced by the bracket, citing the cells.
- [ ] `make eval-baseline` records no regression; `make check` green; reviewers approve.

## Out of scope
- IMSEC, IMCSC, IMSECH and IMCSCH's 709.09 flush point (also inferred; file a follow-up if it is wanted).
- The real SINH and COSH band (T-4423).

## Outcome
- probes/t4425-imtan-imcot-switch (excel-graph, 192 cells) brackets the switch at 2y > 709 / 2y < -708 (y = 354.5 `#NUM!`, 354.500000000001 i; -354 `#NUM!`, -354.000000000001 -i), not at cosh 2y's overflow. All 192 cells pass.
- `tan_saturates` in complex/trig.rs replaces the cosh-finiteness tests in `imtan` and `imcot`; unit test `imtan_imcot_switch_to_i_at_2y_709`. engineering.md updated.
- The bracket matches neither libm's cosh nor e^x overflow, but no constant was fitted: the cut is IMCSCH's existing evidenced `in_csch_negated_band` (T-3881/T-3912), reused as `tan_saturates`, and the new probe brackets it at the 1E-12 input step for all four x.
