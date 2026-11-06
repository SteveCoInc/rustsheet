---
id: T-4576
title: "BINOM.DIST cdf on L140 bup+bgrat at an integer b0: Excel drops bgrat's increment D = x0^a0 near ln D -678, 6.1E-6 relative off the math (t3568-r5-bup-hold A1371)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-4575]
---

## Goal
BINOM.DIST's cdf (and every caller of the same oriented call that Excel treats the same way) is Excel's value where
Excel leaves bgrat's increment D = x0^a0 out of the L140 bup+bgrat composition at c = 1. The cut is modelled from the
mechanism T-4575's probe brackets, not set as a threshold fitted to one cell. Every cell that keeps D stays as it is.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main aabdd668c). This is the only visibly-off cell that
  `make unowned-off ALL=1` reports. It was owned only by deferred T-3712. Its first wrong digit is the 6th, so under
  decision 0267 it is a Tier A cell, and T-3712's deferral note says such a cell is reopened.
- The cell: probes-numeric/t3568-r5-bup-hold Calc!A1371 `=BINOM.DIST(2,1925,0.297025813,TRUE)`.
  - Excel: 7.562827113889096E-290.
  - Ours: 7.562873290726655E-290.
  - 50-digit finite sum (mpmath): 7.5628732907272211E-290. Ours is 7.5E-14 relative off it, Excel 6.1E-6 off.
  - Excel equals the sum less x0^a0 = q^1923 = 4.6176837161387E-295 to 1.3E-13 relative. So Excel returns ret·S without
    the bgrat increment.
  - The oriented call is I_q(1923, 3), q = 0.70297: b0 = 3, n = 2 bup terms, c = 1 (statistics.md "Incomplete beta: the
    integrated model", bullet "L140 bup+bgrat with n >= 2 bup terms"). Excel's own D on the row is a number (B1371
    `BETA.DIST(x0,1923,1,TRUE)` = 4.617683716137408E-295).
- On the same branch Excel keeps D at ln D -595.1 (t3568-r5-bup-hold A1326) and at -487.7 (t3568-r5-bup-tune A1263).
  T-4575 brackets the cut by bup term count and a0. Suspects to test against its values:
  - bgrat's own error exit (`u == 0`, or the `l = w/u` step) at b = 1, where bm1 = 0 and r = z·x^a;
  - x87 intermediates inside bgrat, against Excel's EXP (`x87_exp.rs`);
  - a guard in Excel's composition that skips bgrat when its prefactor is below a fixed binade.

  T-4437's open n = 3 cut at c = 1/2 lists the same suspects. If one mechanism explains both, say so in both tasks.
- Excel is the inaccurate side here, but its error is 6.1E-6, inside decision 0188 R3's G = 1E-4. So Excel's value is
  the target, and this task takes the `visible` stage, not `accuracy`.
- Code:
  - `crates/rustsheet-engine/src/functions/statistical/bratio/bup.rs` `l140_bup_bgrat`;
  - `bratio/bgrat.rs` `bgrat_increment`;
  - `bratio/mod.rs` `kernel_tails`, which BINOM, NEGBINOM and F reach.
  
  Extend the existing path, do not copy it (directive 3). Whether BETA.DIST's own cdf drops D too is T-4575's column B.
  Apply the rule to every caller whose Excel cells show the drop, and only to those.
- Split with T-3712 (deferred): this task owns A1371 only. T-3712 keeps the rest of the t3568 column-A work.

## Decision 0267 cells
- probes-numeric/t3568-r5-bup-hold.case.json (1): Calc!A1371
- The cells of probes/t4575-binom-l140-d-drop where ours and Excel disagree on keeping D (T-4575 lists them).

## Visible-parity stage
Owner of the visibly-off cell probes-numeric/t3568-r5-bup-hold Calc!A1371 and of T-4575's disagreeing cells. The target is
each one within half a unit in the 15th significant digit of Excel's value. Cells where Excel keeps D must stay as they
are.

## Acceptance criteria
- [ ] probes-numeric/t3568-r5-bup-hold Calc!A1371 is within half a unit in Excel's 15th digit. probes-numeric is
      ungated, so this shows as `make unowned-off ALL=1` no longer listing it and the report's M9 probe row losing it.
- [ ] Every cell of probes/t4575-binom-l140-d-drop that disagrees with Excel today on keeping D matches Excel's choice.
      No cell that keeps D changes verdict. Those N known-failure lines disappear from eval/baseline/probes.txt with no
      new ones (N is counted in the Outcome).
- [ ] The cut is a mechanism, written in statistics.md under the L140 n >= 2 bullet with the cells that bracket it.
      Unit tests pin A1371, A1326 and the T-4575 cells on each side of the cut.
- [ ] If anything is chosen by scoring, the decision 0188 R4 guards (reviewer-excel-fidelity check 7) hold:
  - hold-out counts are reported (exact / 4 ulp / 15 digits) on cells not used to choose;
  - a fresh oracle sweep, drawn after the rule was frozen, is scored;
  - the Gate A envelope result is in the Outcome (within 15 digits of Excel, or within 1E-9 of the 50-digit value);
  - the code carries a `FITTED` tag with the fitted range;
  - the reproducibility floor is met;
  - there is no branch that keeps already-matched cells by special case.
- [ ] No gated or probes-numeric cell goes from within 15 digits to visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The exact 0 at the prefactor cut (T-4045, T-4437).
- The rest of T-3712's L140 n >= 2 last digits (deferred).
