---
id: T-3803
title: "BETA.DIST density on the t3568-r5-bup-terms probes: owners for the 11,307 failing density cells (Phase 1b, both within 1.7e-13 of the math)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2586]
campaign: visible
defer: phase-1b
---

## Goal
Every failing cell of probes-numeric/t3568-r5-bup-terms-tune and probes-numeric/t3568-r5-bup-terms-hold, all of them
`BETA.DIST(x,a,b,FALSE)`, is within half a unit of the 15th significant digit of Excel's value, or is moved to a
narrower density task that names its branch (both files name each other), or is listed in the Outcome with Excel,
ours and the 50-digit math and the reason Excel's digits cannot be derived.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Gap triage 2026-11-02 (`make eval` on origin/main 56917e82c, excel-graph). `make unowned-off ALL=1` and
  `make exit-owners MILESTONE=M9` list these two cases as the only BETA.DIST cells with no open owner: 7,026 visibly
  off (all in the M9 rate criterion as `rate:BETA.DIST`) and 4,281 more within 15 digits.
- T-3568 (done) made the probes for its bup-loop hypothesis (docs/research/t2464/r5-bup/RESULTS.md, `mkterms.py`):
  each cell is Excel's density `BETA.DIST(y0, c + j, a0, FALSE)` at a bup point's shapes. They were evidence for the
  bup+bgrat fit, not cells that fit fixes, so neither T-3568 nor its follow-ups (T-3712, T-3719) name them. T-3712 owns
  the bup cells of t3568-r5-bup-tune, -hold and -fresh, not these.
- The triage computed the 50-digit density (mpmath, `x^(a-1)(1-x)^(b-1)/B(a,b)` on the double inputs) for the 7,026
  visibly-off cells. Both Excel and ours are within 1.7e-13 relative of it on every cell. Within 1e-14: Excel on
  3,905 and ours on 4,593. Excel and ours are both within 1e-14 on 3,356. Decision 0188 R1(c) makes this Phase 1b
  evidence, hence `defer: phase-1b`, as for T-3244 and T-3592. They count in the R1(c) rows today only because
  BETA.DIST has no Gate A definition yet (T-3529 adds it). If Gate A's tables show a cell outside the envelope, file
  that cell as its own task without the key.
- Shapes (x from 0.0093 to 0.2993). Each band has an owner task for the density kernel:
  - **min(a, b) > 1 and a + b - 2 >= 90 (10,228 cells: hold 5,164, tune 5,064).** a is 1.0085 to 39, b is 70.4 to
    3,000. This is the Loader saddle point of `loader.rs` `excel_beta_density`. T-3244 (n = 90 to 256), T-3392 (bit
    level) and T-3220 (the port's newly-off cells) are its owners, all waiting on T-2586's model of the Loader
    constant and bd0 series. Most of these cells have n above 256, past T-3244's range.
  - **min(a, b) > 1 and a + b - 2 < 90 (749 cells: hold 353, tune 396).** a is 1.03 to 23.5, b is 17.3 to 90.8.
    Same kernel.
  - **min(a, b) <= 1 (330 cells: hold 167, tune 163).** a is 0.01 to 0.993, b is 17.3 to 3,000.
    `beta_density.rs` `beta_density_small_shape`, owned by T-3035 (visible), T-2821 (bit level) and T-3563/T-3564
    (min < 1 < 8 < max, algdiv).
- Merged from T-3818 (dropped by T-3834, 2026-11-05): T-3818 owned the same two cases whole for their 7,026 visibly-off
  cells, a subset of this task's 11,307, with the same intended fix (the shared density kernel), so this task is the one
  owner. What T-3818 added:
  - Counts (2026-11-02): terms-tune 5,898 cells, 275 bit-exact, 3,602 visibly off; terms-hold 5,967 cells, 283
    bit-exact, 3,424 visibly off.
  - Examples (Excel vs ours, terms-hold): Calc!O231 `BETA.DIST(0.258,14.561,2283.22,FALSE)` 5.537148910195256e-266 vs
    5.537148910194687e-266; Calc!M264 `BETA.DIST(0.0782029528482,13.0,1617.6,FALSE)` 4.0179058036923066e-38 vs
    4.017905803692237e-38; Calc!AA20 7.975405748028135 vs 7.975405748028151 (`(1669435*2^-25+30215771*2^-51)`,
    shape `(28101836*2^-20+53687091*2^-46)`, 720.3).
  - On 9 sampled cells against 50-digit mpmath, Excel is 2.7e-15 to 1.8e-13 relative off the density and the engine
    1.4e-15 to 6.7e-14: Excel is the less accurate one, hence stage `visible`, not `accuracy`.
  - Suspected root cause: Loader's saddle point at a large exponent (the error grows with |ln f|, here up to about 660;
    same family as T-3250 and T-3244) for min(a,b) > 1, and the min(a,b) <= 1 path's lnB (algdiv for b >= 8). T-3569
    found no x87 storage variant of algdiv that fits on the web pins; T-3750 (M14, blocked) asks desktop cells for them.
    Start from whichever of T-3220, T-3244 or T-3250 lands first; do not fit a second density model beside theirs
    (directive 3).
- Cases owned, whole (decision 0136: each case is named on a line with no cell):
  - probes-numeric/t3568-r5-bup-terms-tune
  - probes-numeric/t3568-r5-bup-terms-hold
- probes-numeric is not gated, so no eval/baseline line moves. Measure with `make eval-show CASE=probes-numeric/t3568-r5-bup-terms-hold.case.json` and the report's per-case counts.

## Acceptance criteria
- [ ] Each failing cell of the two cases is within half a unit of Excel's 15th significant digit, or moved to a named
      narrower task (both files name each other), or listed in the Outcome with the three values and the reason.
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring (the -hold case is the hold-out of the -tune case, as T-3568 split them);
      - a fresh sweep after the freeze;
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] No gated known-failure line is added (`make eval-baseline` refuses regressions), and the fit sets of T-3244,
      T-3220 and T-3035 keep their counts.
- [ ] The per-case counts (exact, within 4 ulp, within 15 digits) before and after are in the Outcome, and
      statistics.md's beta density entries record what was found with its hold-out score, or the negatives tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdf cells of t3568-r5-bup-tune, -hold and -fresh (T-3712), and the a0 < 1e-15 band (T-3719).
- The Loader constant and bd0 series themselves (T-2586): this task ports them, it does not crack them.
- The cells T-3244 (both shapes large at n < 300), T-3250 (the a = 1025 column) and T-3220 (the residue at n < 90)
  own in their own cases.

## Visible-parity stage
Target (decision 0114): every visibly-off cell of the two cases within half a unit of the 15th significant digit of
Excel's value. Bit-level residue goes to a follow-up with `defer: phase-1b`.
