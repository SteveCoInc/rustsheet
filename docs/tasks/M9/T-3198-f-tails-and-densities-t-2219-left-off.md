---
id: T-3198
title: "F tails and densities T-2219 left off: F.DIST.RT(3,50,100) on L140 and F.DIST densities at (50,100); T-2219's gen-m9 cells below a + b = 10"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The F cells T-2219 left visibly off, and its gen-m9 cells, agree with Excel to 15 significant digits.

## Context
T-2219 moved half-integer shapes with 10 <= a + b < MAXGAM, outside the closed forms and the ported
bratio branches, to an accurate double-double kernel (`incomplete_beta.rs::half_integer_tail`;
docs/excel-semantics/statistics.md "Incomplete beta in the ironcalc-calc workbooks", bullet
"Accurate tails at half-integer shapes from a + b = 10"). Cells of its table it did not reach:
- probes-numeric/m9-f-dist-rt A19 `F.DIST.RT(3,50,100)`: Excel 1.516786664496894e-6, ours
  1.5167866644968792e-6 (mpmath 1.5167866644968894697e-6): bratio's dispatch is L140 bup + bpser,
  which is ported (`bratio/bup.rs`), so the error is that branch's (T-2506 family).
- probes-numeric/m9-f-dist A26 `F.DIST(0.5,50,100,FALSE)` (Excel 0.08394970207752739, ours
  0.08394970207752746) and A30 `F.DIST(3,50,100,FALSE)` (Excel 1.05292998522241E-05, ours
  1.0529299852224087E-05): the density is Excel's own chain model (`f_density.rs`, T-2586/T-3034),
  which reaches 15 digits only where |c ln x| is small.
- T-2219's 26 gen-m9 cells that stay off moved to T-2221 on 2026-11-03 (see "Triage (2026-11-03)" below).

## Acceptance criteria
- [ ] m9-f-dist-rt A19 and m9-f-dist A26, A30 within half a unit of the 15th digit of Excel.
- [x] Every gen-m9 cell above within 15 digits of Excel, or named in the Outcome and handed to the
      task that owns its branch (T-2221 for Excel's own last bits): all 26 handed to T-2221, triage below.
- [ ] No gated known-failure line added; `make check` green; reviewers approve.

## Out of scope
- The F quantiles (T-2511) and bit-level fits of Excel's formulation (T-2221).

## Triage (2026-11-03)
This is an opus run's triage. The task stays `todo` for the three probe cells. 50-digit references come from mpmath 1.3 at
mp.dps = 50: `betainc(.., regularized=True)` for the tails, and the closed-form density. Each relative error is against that
reference; values are Excel's and ours on main 93a939d2.

| cell | call | Excel rel. error | ours rel. error |
|---|---|---|---|
| gen-m9 2801c1c11c21e134 A374 | `FDIST(3.14159,9,6)` | -1.8e-16 | -1.1e-15 |
| gen-m9 35e8168aae2b00f8 A123 | `F.DIST(100,4,10,FALSE)` | -3.5e-15 | -3.3e-15 |
| gen-m9 3a25a03eb94fcfbd A49 | `FDIST(123456789.123,10,7)` | -1.06e-14 | -1.05e-14 |
| gen-m9 68d8ffb69ff739d8 A273 | `F.DIST.RT(2.5,1E+10,6)` | -1.5e-15 | -2.0e-15 |
| gen-m9 7bd219ec0c0c5e22 A349 | `FDIST(7,10,1E+8)` | +6.7e-15 | -2.1e-15 |
| gen-m9 8dd2312c96effb1b A144 | `FDIST(2.5,8,9)` | -2.1e-16 | -3.5e-16 |
| gen-m9 a5828d3925575713 A235 | `FDIST(42,4,9)` | -1.5e-15 | -1.3e-15 |
| gen-m9 b8ee55934c5b824c A22 | `FDIST(37.5,9,7)` | -3.2e-15 | -2.4e-15 |
| gen-m9 d8266bc15533702a A337 | `F.DIST.RT(1E+10,8,6)` | -1.4e-15 | -1.9e-15 |
| gen-m9 ee2497a5e683c5d9 A118 | `FDIST(3,5,30)` | +3.1e-16 | +7.1e-16 |
| gen-m9 60b0b6e69ad35a45 A151 | `F.DIST.RT(1E+10,9,9)` | +3.8e-15 | +4.7e-15 |
| probes-numeric m9-f-dist-rt A19 | `F.DIST.RT(3,50,100)` | +3.0e-15 | -6.8e-15 |
| probes-numeric m9-f-dist A26 | `F.DIST(0.5,50,100,FALSE)` | +3.2e-16 | +1.2e-15 |
| probes-numeric m9-f-dist A30 | `F.DIST(3,50,100,FALSE)` | +1.03e-14 | +9.0e-15 |

- **gen-m9.**
  - Ours is as close to the reference as Excel is, and usually on the same side. In 7bd219ec0c0c5e22 A349 ours is closer.
  - The other 15 cells take computed arguments and differ from Excel by 1e-16 to 9e-15. None is visibly off.
  - Matching them means fitting Excel's own last bits, not making ours more accurate, so all 26 moved to T-2221's list
    (its "Cells added from T-3198 (2026-11-03)"). The one exception is 2801c1c11c21e134 A374, where ours is 6x further
    from the reference than Excel. It sits at a + b = 15 on the L140 (bup + bpser) path, the same as A19 below.
- **A19.** Ours is 2.3x further from the reference than Excel.
  - The correctly rounded reference is within half a unit of Excel's 15th digit (3.0e-15 against a half unit of 3.3e-15).
  - So routing bratio's L140 dispatch for shapes of 10 <= a + b < MAXGAM to T-2219's double-double kernel
    (`incomplete_beta.rs::half_integer_tail`) would likely meet the criterion.
  - It also moves every cell bup + bpser now matches bit for bit. It needs a full eval and the R4 guards if any cut is
    chosen by scoring, so it was not attempted in this run.
- **A26.** Ours is 1.2e-15 off the reference and Excel 3.2e-16 off, inside Excel's chain-model density (`f_density.rs`).
- **A30.** Both are about 1e-14 off on the same side, so it is likely the chain model's own error.

## Progress (2026-11-03, evidence PR)
This opus run measured the L140 bup+bpser branch and probed Excel at its F points. It changed no code, so the three
probe cells stay open. The finding is in statistics.md, "Incomplete beta L140 bup+bpser at the F points".
- New probe: probes-numeric t3198-f-l140-args, 414 rows, on excel-graph. Its visibly-off cells A1:F414 are this task's
  (columns E and F are the F tails; C and D are BETA.DIST at u and v).
- The F argument form is not the main error. (u, v) stays the best of four forms on every ported branch. A19
  (`F.DIST.RT(3,50,100)`) is the exception: Excel there equals `BETA.DIST(0.4,50,25,TRUE)`, at y0 = 0.6, not
  u = 0.6000000000000001.
- The kernel is the main error: our L140 bup+bpser is bit-exact on 596 of Excel's 816 BETA.DIST values at these points.
  The scoring was run in throwaway tests on the probe's columns C and D and the bratio fixture's L140 rows:
  - T-3568's power sum `Σ R_i·EXP(i·LN y0)` gets 669 on the probe and 99 of 122 on the fixture (91 today).
    `make eval-check` gives 14 gated cells fixed, in gen-m9, ironcalc-calc and probes. It also gives 1 gated
    regression, ironcalc-calc BETADIST_BETAINV.xlsx H32 (`BETA.INV` at (5, 5) crosses this kernel).
  - The same power sum fails two unit tests that hold Excel values at x0 = 0.5: `l140_whole_b0_takes_ln_beta_of_one`
    (`BETA.DIST(0.5,100,10,TRUE)`) and `ported_tails_rise_with_x_where_excel_does` ((10, 10) at 0.5).
  - Correctly rounded powers (libm `pow`) get 636 and 102. They keep H32 and x0 = 0.5, but move
    `BETA.DIST(0.1,100,10,TRUE)` 2 ulp off.
- Next step. Find the power Excel takes in bup's sum: exact at x0 = 0.5, and EXP·LN-like elsewhere. Candidates are an
  x87-extended EXP(i·LN y0) stored once, or a cut on n.
  - Choose it with decision 0188's R4 guards: fix the hold-out half of the probe rows before scoring, then a fresh
    excel-graph sweep.
  - If BETA.INV still moves H32, keep its search on today's recurrence, as T-3561 did for L140 bup+bgrat
    (`bratio::beta_search_tails`).
  - Then re-measure A19 at the chosen kernel.

## Progress (2026-11-03, power-sum PR)
This opus run moved L140 bup+bpser's bup sum to T-3568's power sum (`bratio/bup.rs` `l140_bup_bpser`,
`BupSum::Powers`). The three probe cells stay open, so the task stays `todo`.
- Scoring (decision 0188 R4.1, a structural model ranked from candidates; no fitted constant, so no `FITTED` tag). The
  population is the 816 BETA.DIST cells (columns C and D) of t3198-f-l140-args on L140 bup+bpser. The hold-out split was
  fixed before any scoring: even rows train, odd rows held out. That holds out 50%, more than R4.1's ~30%, so the
  hold-out is stricter, not looser.
  - No numeric margin was written down before scoring. The criterion fixed beforehand was that the power sum must beat
    both the recurrence and correctly rounded powers on the hold-out half in every column. It does, and its hold-out
    score is above its train score.
  - Cells per half (bit-exact / within 4 ulp / within 15 significant digits), each half of 408:

    | candidate | train | hold-out |
    |---|---|---|
    | TOMS recurrence (main) | 302 / 405 / 394 | 299 / 403 / 396 |
    | power sum `Σ R_i·EXP(fl(i·LN y0))` (chosen) | 333 / 407 / 400 | 341 / 407 / 404 |
    | correctly rounded powers (libm `pow`) | 319 / 406 / 397 | 319 / 406 / 400 |

  - Five other forms of the power, and exact powers at y0 = 0.5 only, scored lower in bit-exact cells on both halves
    (statistics.md lists them). No cut on n wins. Bratio fixture: 91 to 99 of 122 (ratchet raised).
- Fresh sweep after the freeze (R4.2): new probes-numeric t3198-l140-power-sweep, 400 seeded random BETA.DIST points on
  the branch (excel-graph). 259 bit-exact against 244 for the recurrence, and 149 for correctly rounded math.
  - Gate A envelope: 400 of 400 inside on this head and on main, so none leaves (`make eval-gates` and the case's
    `.math.json`, checked by the fidelity review).
  - The sweep is uniform, not log-uniform, because the branch's domain is bounded: both shapes above 1, b0 < 40 and
    b0·x0 > 0.7. Shapes drawn in (1, 60] and x in [0.02, 0.98], filtered by the engine's dispatch to the branch, cover
    every b0 band of it. Shapes past 60 leave the branch through b0 >= 40 or bratio's swap.
- Gated eval: 13 cells fixed (gen-m9 4, ironcalc-calc 8, probes 1), no regression. BETA.INV's search keeps the recurrence
  (`BupSum::Recurrence`, `bratio::beta_search_tails`): crossing the power sum moved ironcalc-calc BETADIST_BETAINV.xlsx H32
  to 0.5550000000000002. That is the only cell the choice moves in any suite, so the evidence is one cell (T-3947
  revisits it).
- Ungated probes-numeric, as the fidelity review measured it: 508,502 bit-exact cells become 509,493, and visibly-off
  cells (outside 15 digits) drop from 84,722 to 84,706. The change is gross, not uniform. It newly misses cells in
  t0969-beta-prefactor (Pref!D22), m9-beta-dist-a100 (A39), t1745-beta-inv-roundtrip (6), t2145-binomial-sweep (14),
  t2145-negbinom-sweep (5), t2220-f-dist-sweep-cdf and -rt (about 11 each), t2464-w2f-absweep and -xruns-1 (dozens each),
  -xruns-2 (6), t3198-f-l140-args (about 60 in columns C to F), t3198-l140-power-sweep (29), t3352-binom-negbinom-sweep
  (A86) and t3567-r5-fdf1-hold (11). Each of these cases is net positive. No cell is kept by a band.
- Two unit expectations at y0 = 0.5 lose: `BETA.DIST(0.5,100,10,TRUE)` is 1 ulp high, and (10, 10) at 0.5 is not monotone
  against the next double. Both stay in their tests as known-miss assertions of today's value, marked `TODO(T-3947)`,
  so nothing is deleted or skipped. T-3947 also takes the sweep's 17 cells more than 4 ulp off.
- A19 `F.DIST.RT(3,50,100)`: 1.5167866644968792E-06 becomes 1.5167866644968803E-06. That is closer to the 50-digit
  reference than Excel (-6e-16 against +3.0e-15) but still not Excel's 15th digit.
- Left for the next run:
  - A19: Excel equals `BETA.DIST(0.4,50,25,TRUE)` at y0 = 0.6, not at u = 0.6000000000000001. That is the F argument
    question, and one cell is not enough evidence to change the argument rule.
  - A26 and A30: `f_density.rs`.

## Landing notes
- PR #2997, branch agent/functions-stat/T-3198-f-l140-power. CI was green on 02173ea6, and this push restarts it.
- Reviewers on 02173ea6: design APPROVE, security APPROVE, excel-fidelity REQUEST_CHANGES. This push answers fidelity's
  three blocking findings:
  - the full R4.1 table (bit-exact, 4 ulp and 15 digits for three candidates on both halves), the margin statement and
    why a 50% split is acceptable;
  - the sweep's Gate A envelope result;
  - the known-miss assertions that replace the deleted A39 expectation and the skipped monotonicity pair.
  It also covers the MINORs: disclosure of the ungated losses, the single-cell evidence for BETA.INV's recurrence, why
  the sweep is uniform, and T-3947's wording.
- Next step: re-run reviewer-excel-fidelity on the new head, and design and security too, since the head moved. Once CI
  is green and all three approve, mark the PR ready and enqueue it (`enable_pr_auto_merge`, SQUASH).
- Do not repeat: deleting or skipping a test expectation. Fidelity rejected that without a decision record.
