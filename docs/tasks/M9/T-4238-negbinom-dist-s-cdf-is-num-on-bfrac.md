---
id: T-4238
title: "NEGBINOM.DIST's cdf is #NUM! on bfrac at number_s 1E+20"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`NEGBINOM.DIST(f, 1E+20, p, TRUE)` at 40 or more failures (bratio's bfrac branch) is `#NUM!` wherever Excel's is, from
a rule backed by a probe, recorded in statistics.md.

## Context
- Follow-up of T-3576, which found the L140 rule (bup's ratio product overflows) and left bfrac's cells:
  probes/t3576-negbinom-cdf-huge-s Calc!A50, A51 and A52 (`NEGBINOM.DIST(40|100|1000,1E+20,0.323212154487823,TRUE)`)
  are `#NUM!` in Excel; ours 0. bfrac at number_s 1E+15 and below is 0 at 40, 100 and 1000 failures (A22:A24,
  A36:A38, A54:A58).
- statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s (T-3553)", "Still open".
- At a0 = 1E+20 and b0 = 41, a0 + b0 rounds to a0 (ulp 16384), and TOMS brcomp's z underflows to 0, so TOMS bfrac
  returns 0 at once. Excel's bfrac prefactor is its own beta density times x·y (statistics.md "Incomplete beta L140
  and bfrac: the prefactor is Excel's own density times x·y"): the density at that shape is the first suspect.
- Code: `crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs`, `bratio::l140_overflows`'s neighbour.
- Also there: probes/t3576-negbinom-cdf-huge-s A75 (`NEGBINOM.DIST(28,999999999999999,0.99999999999999,TRUE)`, bfrac
  on the swapped call) is 1 ulp low (0.9999992473499034 against Excel's ...035): Tier B, take it only if the rule's
  work reaches it.

## Acceptance criteria
- [x] A probe brackets the number_s cut between 1E+15 and 1E+20 on bfrac (and BETA.DIST at the same shapes), and
      statistics.md records the rule or the bracket.
- [x] The engine matches probes/t3576-negbinom-cdf-huge-s A50:A52.
- [x] `make check` green; reviewers approve.

## Out of scope
- L140's overflow (T-3576, done).

## Outcome
- Rule (structural, no constant): on bratio's bfrac branch Excel's cdf is `#NUM!` where the oriented first shape absorbs
  the second, fl(a0 + b0) = a0 after the swap. BETA.DIST's density is `#NUM!` where alpha absorbs beta with both
  shapes above 1 (its Loader branch). Code: `bratio::first_shape_absorbs`, `bratio::bfrac_shapes_absorbed`, called
  from BETA.DIST's cdf and density and NEGBINOM.DIST's cdf.
- Evidence, three rounds, each with predictions committed in `docs/research/t4238/` before its excel-graph fill:
  1. probes/t4238-negbinom-bfrac-huge-s, 126 cells, H_absorb matched all 126 (number_s 1E+15 to 1E+20; cuts at
     5E+17/6E+17, 1E+18/2E+18, 9E+18/1E+19).
  2. Round 1's rule (either shape absorbed) failed Gate A (accuracy/BETA.DIST s1 A183, A190 and r4-T-3879 A181, A182:
     Excel 0 where the first shape is absorbed) and t3007 A6 (a shape below 1). Narrowed to the oriented first shape,
     then probes/t4238-r2-absorb-orientation (38 cells): every cell separating H_oriented from H_raw matched H_oriented.
  3. r2 A12, A13 (x0 0.999) broke it, so probes/t4238-r3-absorb-whole-shape (12 cells) tested H_frac (whole b0)
     against H_x (x0 near 1): H_x on all 12, x0 0.99 `#NUM!` and 0.999 a value.
- Not implemented: the x0 cut in (0.99, 0.999). No mechanism is known, and a threshold would be a fit. **Disclosed
  loss (decision 0082's precedent):** main matched Excel at six x0 0.999 / 0.001 cells (r2 A12, A13; r3 A1, A2, A6,
  A10: 0 or 1), and this rule, which has no x0 cut, makes them `#NUM!`. They are new-case known failures owned by
  T-4824, whose acceptance restores them. Net on Excel-evidenced cells: 4 gated fixes (t3576 A50:A52, t3007 A20), Gate
  A BETADIST A184, 54 more cells of the new probes; 6 lost. The fidelity review's per-cell diff against the merge base
  found no other changed cell. r2 A14, B14 are
  close-digit Tier B, also noted in T-4824.
- Eval: probes/t3576-negbinom-cdf-huge-s A50:A52 pass (88/89; A75 stays Tier B), t3007 A20 fixed and A6 kept,
  t4238 126/126, r2 34/38, r3 8/12; Gate A: BETADIST s1 A184 (T-4733's, alpha 2.2E+114 absorbing beta 2.0E+66) now
  passes and leaves known-failures.tsv, A181:A183 and A190 still pass; no regressions.
- statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s" records the rule, the orientation, the density and the
  open x0 cut.
