---
id: T-3472
title: "F.DIST and F.DIST.RT cdf, and BETA.DIST(0.5,100,100,TRUE): 6 cells where Excel is within 1.6 ulp of the math and ours is 3.6 to 13.8 ulp off (accuracy)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
defer: phase-1b
---

## Goal
The 6 incomplete-beta cdf cells below (5 F.DIST / F.DIST.RT on the T-2220 sweeps and BETA.DIST(0.5,100,100,TRUE)) are
within half a unit of the 15th significant digit of Excel's value, through a more accurate kernel on the bratio branch
each takes (not a fit to Excel's bits). No cell of those cases that is within 15 digits today moves off.

## Context
- Gap triage 2026-10-31i (`make eval` on origin/main 0c2fbf706, excel-graph; `scripts/exit_owners.py --all-off` and
  `--milestone M9`, `rate:F.DIST`, `rate:F.DIST.RT`, `rate:BETA.DIST`): no open owner, deferred or not. They are the
  cells of that unowned group where Excel is close to correctly rounded; the rest (Excel itself 3 to 620 ulp off the
  math) are T-3474's and T-3592's, Phase 1b.
- Signed ulps from mpmath at 60 digits (F as I_x(d1/2, d2/2) with x = d1·X/(d1·X + d2), the right tail as
  I_y(d2/2, d1/2)), Excel / ours:
  - t2220-f-dist-sweep-d2-1 A400 `F.DIST(0.24,7,1,TRUE)`: Excel 0.0805651348300428 (-0.2), ours 0.08056513483004273 (-5.3)
  - t2220-f-dist-sweep-rt A467 `F.DIST.RT(3,5,10)`: Excel 0.06555756209384413 (+1.2), ours 0.06555756209384406 (-3.8)
  - A504 `F.DIST.RT(1,7,1)`: Excel 0.6493833371797926 (+1.1), ours 0.6493833371797919 (-5.2)
  - A954 `F.DIST.RT(1,100,7)`: Excel 0.5640919724091795 (+1.6), ours 0.5640919724091789 (-3.8)
  - A984 `F.DIST.RT(1,100,50)`: Excel 0.5108794921920699 (+0.6), ours 0.5108794921920683 (-13.8)
  - m9-beta-dist-a100 A51 `BETA.DIST(0.5,100,100,TRUE)`: exactly 1/2 by symmetry; Excel 0.4999999999999999 (one
    double below), ours 0.5000000000000004 (two doubles above)
- Root cause (suspected, one kernel, several branches): every caller makes one bratio call (statistics.md
  "Incomplete beta is TOMS 708 `bratio`"). By the dispatch, A400 swaps to a0 = 0.5, b0 = 3.5, x0 = 0.373 (w1 =
  bpser(b0, a0, y0) at x0 >= 0.3); A467 swaps at the mean to a0 = 5, b0 = 2.5, x0 = 0.4 (L140: bup with n = 2 then
  bpser, x0 <= 0.7); A504 swaps to a0 = 0.5, b0 = 3.5, x0 = 0.125 (bup with n = 20 then bgrat); A954 and A984 have
  a = 50 against b = 3.5 or 25, near the mean (check the swap, then L140 bup+bgrat); A51 is a0 = b0 = 100, bfrac's b0 >= 40.
  Confirm each with `bratio/mod.rs` `Branch` before changing anything. F goes through `fisher.rs` (closed forms at a
  df of 2, then `bratio::closed_or_ported_tails` / `kernel_tails`, T-2847), and a branch the port has not taken yet
  may still fall back to `incomplete_beta.rs`, so the first question is which code path each cell takes today.
- Related open work: T-3382 (accurate bgrat increment, L140 one bup term), T-3665 (a0 < 1 bup+bgrat at x0 in
  [0.29, 0.3)), T-3383 (bfrac and bup arithmetic). Share pieces with them rather than duplicating (directive 3).
- Excel is near the math, so the target is the math to about 1 ulp. If a cell turns out to need Excel's own rounding,
  it moves to a `visible` follow-up with `defer: phase-1b` (decision 0188 R5) and the R4 guards of
  reviewer-excel-fidelity check 7.
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/, `fisher.rs`, `incomplete_beta.rs`.

Owned cells (all cdf or right tail):
- probes-numeric/t2220-f-dist-sweep-d2-1.case.json (1): Calc!A400
- probes-numeric/t2220-f-dist-sweep-rt.case.json (4): Calc!A467, A504, A954, A984
- probes-numeric/m9-beta-dist-a100.case.json (1): Calc!A51

Cells added by gap triage (2026-11-03, `make eval` on origin/main c2062e5c8, excel-graph): four gen-m9 F cells that
`scripts/exit_owners.py --all-off` lists as visibly off and owned only by deferred T-2221 ("Cells added from T-3198",
which judged them within 15 digits; the report does not). gen-m9 is a Gate B suite, so FDIST is below Gate B's rate
until they pass. Each is an accuracy cell: Excel within 2 ulp of the 50-digit math (mpmath 1.3, dps 50, `betainc` at
the double arguments), ours 2 to 8 ulp off, and the correctly rounded value is within half a unit of Excel's 15th
digit. Relative errors Excel / ours, and the branch `bratio::dispatch` gives (F as I_x(d1/2, d2/2), right tail
I_v(d2/2, d1/2)):
- gen-m9/2801c1c11c21e134 Calc!A374 `FDIST(3.14159,9,6)`: Excel 0.0884857396733851, ours 0.08848573967338502;
  -1.8E-16 / -1.12E-15; I_v(3, 4.5) at v = 0.175, `L140BupBpser` (A467's branch; T-3198's L140 power-sum work may
  move it first, re-measure after it lands).
- gen-m9/285d87cea1aaa73c Calc!A34 `FDIST(40,7,MINUTE(0.2))` = `FDIST(40,7,48)`: Excel 6.448979666396885E-18, ours
  6.448979666396879E-18; -3E-17 / -9.9E-16; I_v(24, 3.5) at v = 0.146, `BpserLarge` (b0·x0 = 0.51).
- gen-m9/3a25a03eb94fcfbd Calc!A181 `F.DIST(Data!$B9,MONTH(17720),1,TRUE)` = `F.DIST(1,7,1,TRUE)`: Excel
  0.3506166628202075, ours 0.35061666282020815; -1.3E-16 / +1.77E-15; swapped to a0 = 0.5, b0 = 3.5, x0 = 0.125,
  `BupBgrat`: the complement of A504 `F.DIST.RT(1,7,1)` above, so one fix should land both.
- gen-m9/b6593d7dd13f2dac Calc!A50 `FDIST(Data!$E6,5,1.5)` = `FDIST(1.005,5,1)` (Excel truncates the df): Excel
  0.6356868721336002, ours 0.6356868721336008; -3.9E-16 / +4.9E-16; a0 = 0.5, b0 = 2.5, x0 = 0.166, `BupBgrat`.
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A374
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A34
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A181
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A50

## Visible-parity stage
Decision 0083 / 0114 (`accuracy`): met when the 10 cells above (6 probe cells and 4 gen-m9 cells) are within half a unit of the 15th significant digit
of Excel's value, and no cell of the three cases that is within 15 digits today moves off. Bit-level criteria still
unmet then move to a follow-up with `defer: phase-1b`.
- Split (T-3229 check): 5 of the cells below are also listed by T-3592, which owns them; this task keeps the rest.

## Acceptance criteria
- [ ] The 10 cells above (6 probe cells and the 4 gen-m9 cells added 2026-11-03) are within half a unit of the 15th significant digit of Excel's value (`make eval-show` output
      for each case in the Outcome), with the branch each takes named.
- [ ] On t2220-f-dist-sweep-d2-1, t2220-f-dist-sweep-rt and m9-beta-dist-a100 the count within 15 digits of Excel does
      not fall, and no probes-numeric or gated cell goes from within 15 digits to visibly off; any that does is listed
      with Excel / new / exact values, or the change is narrowed.
- [ ] Unit tests pin the fixed cells at Excel's value, with the case and cell named.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F.DIST density (T-3173, T-3174, T-3474) and every cell where Excel is off the math (T-3474, T-3592; Phase 1b).
- F.INV and F.INV.RT (T-3398), which invert whatever cdf this lands.

## Progress notes
2026-11-03, opus run (findings; task deferred to Phase 1b, no engine change). Every one of the 10 cells is already
inside Gate A's envelope (ours 2 to 14 ulp from the 50-digit math, far within SPEC-0029's 1e-9), so this is
last-digit work on dense-sweep F and BETA.DIST cells, which decision 0188 R5 parks in Phase 1b: `defer: phase-1b`.
What was measured, so the Phase 1b run does not repeat it (scratch, not committed; full `rustsheet-eval run` on
origin/main d8640a46, compared cell by cell against the same run without the change):
- **Branches today** (`bratio::dispatch` on `fisher.rs` `f_beta_arguments`; two Context guesses were wrong):
  A400 `W1Bpser` swapped (a0 0.5, b0 3.5, x0 0.373; the split-z bpser, T-3879); A467 and gen-m9 A374
  `L140BupBpser` swapped (a0 5, b0 2.5, x0 0.4; a0 3, b0 4.5, x0 0.175); A984 `L140BupBpser` unswapped (a0 50,
  b0 25, x0 2/3: 24 bup terms of T-3198's power sum plus bpser(50, 1, x0)), not bup+bgrat; A954 `Bfrac` swapped
  (a0 3.5, b0 50, x0 0.0654), not L140; A51 `Bfrac` at the mean (a0 = b0 = 100, outside T-3571's Lentz region,
  so TOMS's recurrence); gen-m9 A34 `BpserLarge`; A504, gen-m9 A181 and A50 `BupBgrat` at a0 = 1/2, which F does
  not port (`half_bgrat` false), so they are Cephes (`beta_reg_tails`).
- **F through bup + `bgrat_half` on `BupBgrat`** (the BETA.DIST path, every a0 = 1/2 point): A504, gen-m9 A181 and
  A50 land (0.6493833371797926, 0.3506166628202074, 0.6356868721336002), but net it is even: gen-m9 6 visibly-off
  cells come within 15 digits and 6 go off, two of them bit-exact today (gen-m9 7c20351562a2d71d A205 and
  c18dbf21f11093f3 A151, `FDIST(x,1,9)`: 0.19094711716972382 against Excel's 0.190947117169724, so
  `make eval-baseline` refuses it); probes-numeric 22 on, 21 off (t2220 cdf/rt/d1-1/d2-1, t3567-r5-fdf1-hold).
  This is T-2510's result again, unchanged by T-3382/T-3567. On the d2 = 1 cells mpmath puts Excel itself 5 to 12
  ulp off (`F.DIST(0.24,20,1,TRUE)` -11.8, `F.DIST(0.621,20,1,TRUE)` -12.2), our bgrat_half -6 to +10: Excel's
  bgrat there is neither ours nor the math (T-3474's cells).
- **bfrac's error, decomposed** (mpmath 1.3, dps 50): A954 brcomp (Excel's density times x·y) +4.3 ulp, TOMS's
  double recurrence r -10.0 ulp, w -3.6 ulp; A51 brcomp +0.03, r +5.7, w +4. With the same recurrence in double-double
  (exact λ and y = 1 - x0, stop at 1e-3·eps) both land (0.5640919724091793, 0.49999999999999994), but taken on
  every non-Lentz bfrac point it moves 2,300 cells: probes-numeric 45 within 15 digits, 23 off (t3953 tiny bfrac
  tails, t2220 rt at df1 100, t1530 at df 1E10, BINOM.DIST(66,100,0.7,TRUE)), and 5 bit-exact gated cells to
  1 ulp (probes t2435-dist-extremes A35, A36, A38 `FDIST(1,6,1E+10)`; gen-m9 7c20351562a2d71d A254,
  f2bd5f380ec002bf A29). Excel's fraction is neither TOMS in doubles nor the math, so any narrowing is a fit
  (R4 guards). T-3571's Lentz loop also lands both cells (0.5640919724091793, 0.49999999999999967) but is
  `FITTED` only for the unswapped call at z in [-8, -0.2], which neither point is.
- The L140 bup+bpser, split-z bpser and BpserLarge cells sit in the residual of bit-exact models (T-3198's power
  sum: 674 of 816 exact; T-3879: 1,139 of 1,139), so moving them toward the math costs exact cells; not tried.
What Phase 1b would do next: an R4 study of Excel's bfrac arithmetic off the Lentz region (swapped calls with a0 < 40,
and |z| < 0.2), with A954 and A51 among its cells, and the d2 = 1 `BupBgrat` cells with T-3474.
