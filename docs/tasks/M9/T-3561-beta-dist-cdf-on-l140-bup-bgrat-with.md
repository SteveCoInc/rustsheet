---
id: T-3561
title: "BETA.DIST cdf on L140 bup+bgrat with two or more bup terms: BETADIST_BETAINV G5 is 2.5E-14 off where Excel is correctly rounded"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3382]
campaign: accuracy
---

## Goal
BETA.DIST's cdf on bratio's L140 bup+bgrat branch with two or more bup terms (bgrat orientation b0 from 2 to 40,
a0 > 15, x0 > 0.7) is accurate to the last digit or so, so the gated cell ironcalc-calc BETADIST_BETAINV.xlsx G5,
where Excel is correctly rounded, is within 15 digits of Excel or bit-exact. Today that branch falls back to the
old Cephes kernel, which loses about 160 ulp there.

## Context
- Gap triage 2026-11-01 (`make eval` on b98b3c00d). The M9 exit row's ironcalc-calc measure is not met: 44 R1(c)
  visibly-off cells against an allowance of 36. `scripts/exit_owners.py --all-off` lists three of them as owned by
  a `defer:` task only, BETADIST_BETAINV G5, H5 and H26. T-3374 names G5 and G26 only in its Out of scope line, so
  in practice nobody owns them. This task takes G5. T-3562 takes H5 and H26, and needs G5 first.
- The cell:
  - ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx (1): Sheet1!G5
  - `_xlfn.BETA.DIST(33587,3,600,TRUE,2000,5489582)`: Excel 0.6734553797493481, ours 0.6734553797493648 (2.5E-14
    relative, about 170 ulp).
  - At 50 digits (mpmath, this triage) the value is 0.67345537974934809577. Excel is correctly rounded (1.3E-17), as
    docs/excel-semantics/statistics.md says ("Two stay far": G5 is the correctly rounded 0.6734553797493481, and
    Cephes' fraction loses about 160 ulp at t = 0.0058, reflected to I_0.994(600,3)).
- Measured in the engine on this triage's scratch case: `BETA.DIST((33587-2000)/(5489582-2000),3,600,TRUE)` gives the
  same 0.6734553797493648. So the (x - A)/(B - A) scaling is not the cause. One ulp of t moves the value by about
  1.5E-16, a hundredth of the error. `1-BETA.DIST(1-t,600,3,TRUE)` gives 0.6734553797493659.
- Why it falls back: `bratio::dispatch` (crates/rustsheet-engine/src/functions/statistical/bratio/mod.rs) gives
  (a, b, x) = (3, 600, 0.00576) λ < 0, so it swaps to (a0, b0, x0) = (600, 3, 0.994). That is b0 < 40,
  b0·x0 > 0.7, x0 > 0.7 and a0 > 15, so the branch is `Branch::L140BupBgrat`. TOMS 708 then takes n = 2 bup terms
  (b0 = 3 is an integer, so n = 3 - 1 and b0 = 1) and then bgrat. `ported_tails_with` returns `None` for that branch,
  and `beta_dist_lower_tail` keeps today's kernel.
- T-3382 ports L140 bup+bgrat with one bup term (1 < b0 + 1 < 2): RN(W + D), with W from the beta density in bup's
  orientation and D an accurate bgrat increment, in extended precision. Its Out of scope leaves "n > 1 bup terms
  (2 <= b < 40)". This task is that remainder, so it extends T-3382's composition (directive 3: one bup+bgrat path,
  not a second one) and waits for it. T-3468 item 1 ("n > 1 bup terms") is the research side. If T-3468 finds Excel's
  own n-term W first, use it.
- Stage `accuracy`: Excel is the correctly rounded side here, so the fix is an accurate kernel (an n-term bup sum
  plus an accurate increment), not a fit to Excel's bits. Before widening the branch, measure Excel against 50-digit
  math on the eval cells that reach it. Those are the cells `bratio::dispatch` sends to `L140BupBgrat` with
  b0 >= 2, which BETADIST_BETAINV.xlsx, BETA_GAMMA.xlsx, the m9-beta-dist-a* probes and gen-m9 hold (list them in the
  Outcome). Where Excel is not near correctly rounded, that is visible-stage work for a follow-up, not this task.
- Callers: only BETA.DIST (`beta_dist_lower_tail`) and BETA.INV's search (inverse_beta.rs) are in scope. F, BINOM and
  NEGBINOM reach L140 through `kernel_tails` and stay there (fisher.rs: F at a df of 1 moved gated cells when bgrat
  changed, T-2510).

## Acceptance criteria
- [x] `bratio::ported_tails_with` covers `Branch::L140BupBgrat` with n >= 2 bup terms (b0 from 2 to 40) for
      BETA.DIST through T-3382's composition, extended (no copied bup or bgrat code). Against mpmath on a generated
      sweep of the branch (a0 15 to 1E4, b0 2 to 39.9, x0 0.7 to 0.9999), the Outcome reports the counts of
      results within 1, 4 and 16 ulp of the correctly rounded value. The script is committed under
      docs/research/t3561/.
- [x] This known-failure line disappears from eval/baseline/ironcalc-calc.txt with no new one in any baseline:
      `F	ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx	Sheet1!G5`. If G5 ends within 15 digits but not bit-exact,
      the line stays and the Outcome says so. The visible-parity target below is what counts.
- [x] No cell of any suite goes from within 15 digits to visibly off. Every cell that changes on the branch (fixed,
      lost bit-exact, closer, farther) is listed in the Outcome from a main-against-HEAD `make eval`.
- [x] No fitted constant. If a structural choice (operation order, where to store an intermediate) is picked by
      scoring Excel cells, decision 0188 R4 applies (reviewer-excel-fidelity check 7): hold-out counts for the
      chosen and the correctly rounded candidate, a fresh sweep after the freeze, the Gate A envelope result, the
      reproducibility floor (no target below the 4-ulp web/desktop floor), and no keep-matched branch.
- [x] A unit test pins G5 (`BETA.DIST(33587,3,600,TRUE,2000,5489582)` = 0.6734553797493481), with the cell named.
- [x] `make check` green; reviewers approve.

## Out of scope
- BETA.INV H5 and H26 (T-3562), and G26, which is within 4 ulp (T-3374).
- One bup term (T-3382), F, BINOM and NEGBINOM callers of L140, and Excel's density to the bit (T-3468).

## Visible-parity stage
Decisions 0083 and 0114 (`accuracy`): done when G5 is within half a unit of the 15th significant digit of Excel's
value through a more accurate kernel. A fit to Excel's bits does not count. Bit-level residue on the branch moves to
a follow-up with `defer: phase-1b`.

## Outcome
BETA.DIST's cdf takes L140 bup+bgrat with n >= 2 bup terms (and one term at c = 1, b0 = 2) through T-3382's
composition, extended rather than copied: `bratio/bup.rs` `l140_bup_bgrat` is RN(fl(ret·S) + D) for every n. ret is
T-3382's register prefactor on the engine's density, S = `bup_power_sum` is T-3568's Excel form 1 + Σ fl(R_i·EXP(fl(i·LN y0)))
(1 at n = 1, so T-3382's cells keep their bits), and D is `bgrat_increment` for c < 1 or `power_increment`, x0^a0 in
double-double from the exact y0, for c = 1. T-3568 (the research side T-3468 split off) found Excel's own n-term W, so
this task takes it, as its Context says. TOMS bup's k is one helper now (`bup_break_after`), shared with `bup`.
The port is narrowed to the evidenced region. At n >= 2 or c = 1 it needs n <= 9 (`MAX_BUP_TERMS`: T-3568's residual
grows from n = 10, and every cell of its probes that left 15 digits had n >= 19) and a0 <= 3000 (`MAX_BUP_A0`:
T-3568's probe range; at a0 = 5E9, probes-numeric t2528-f-dist-huge-df2 A4 and A16 went from 4 ulp to visibly off).
It also needs a normal density and x0^a0 within e^-650. Points that fail any of these keep today's kernel (a first
cut without those guards returned 2.7E-308 for 8E-320 and 0 for 1.1E-275). BETA.INV's search keeps today's kernel on
the n-term branch, because crossing the new cdf took gated BETADIST_BETAINV H28 from Excel's bit to 1 ulp
(follow-up T-3867). T-3712 keeps F, BINOM and NEGBINOM; its Context now says so.

- G5 is bit-exact (0.6734553797493481). Its known-failure line is gone from eval/baseline/ironcalc-calc.txt and no
  baseline gained a line (`make eval-baseline`: 0 regressions). Unit test `l140_many_term_bup_bgrat`.
- Sweep against 50-digit mpmath (`docs/research/t3561/sweep.py`, seed 35610: 1,000 points on the branch, a0 in
  (15, 1E4], b0 in [2, 39.9], y0 = 1 - x0 log-uniform in [1E-4, 0.3], half swapped, cdf strictly in (0, 1)).
  Exact / within 1 / 4 / 16 ulp of the correctly rounded value: main 212/247/295/398, HEAD 215/280/344/476.
  161 points get closer, 52 farther (worst 135 -> 710 ulp at a0 = 1494, b0 = 3: Excel's W tracks its density's
  error at large a0, T-3568), and 787 stay the same.
- Eval, main against HEAD (`make eval` on both, every suite). Only these cells change, and none goes from within 15
  digits to visibly off:
  - ironcalc-calc BETADIST_BETAINV G5: visibly off to exact. H5 and H28 do not change.
  - probes-numeric t3568-r5-bup column A, the 950/950/475 BETA.DIST cells (exact / within 4 ulp / within 15
    digits): tune 12/54/146 -> 70/209/299, hold 18/69/139 -> 76/228/302, fresh 14/40/86 -> 45/124/173. Moves that
    lose ground: fresh A230 (4 ulp -> 15 digits), hold A68 (exact -> 4 ulp), hold A586 (exact -> 15 digits), tune
    A838 (exact -> 4 ulp). Column A's F, BINOM and NEGBINOM rows do not move.
  - m9-beta-dist-a100 A23 and A35, -a2 A49, -a5 A43: 4 ulp -> exact. -a10 A49, -a100 A29 and A41, -a50 A29 stay
    within 4 ulp.
  - t1745-beta-inv-roundtrip (the BETA.DIST of a BETA.INV): G150, G151, G158, H150, H158, I159 visibly off -> 4 ulp;
    G159 visibly off -> 15 digits; G152, H159, I150, I151, I158 15 digits -> 4 ulp; H100, I134 4 ulp -> exact; H151
    within 4 ulp; H134, I152 exact -> 4 ulp.
- R4 (decision 0188): no constant is fitted. The composition is T-3568's, frozen before its hold-out (its scores and
  guards are in its Outcome). The two caps are ranges taken from T-3568's RESULTS and probe generator, not scores.
  The n cap was checked against the cells it removes (all n >= 19), so it is reported here as a structural choice.
  The port's own scores on t3568-r5-bup-hold (hold-out) are 76/228/302 of 950, against 18/69/139 on main. Gate A
  envelope (within 15 digits of Excel or within 1E-9 of the 80-digit math sidecar): 0 points outside on fresh and
  hold, worst relative error against the math 1.1E-13 (fresh) and 1.7E-13 (hold). No target is below the 4-ulp
  floor, and there is no keep-matched branch.

Eval delta: ironcalc-calc 1 known failure fixed (G5). Non-gated probes-numeric: about 470 more cells within 15
digits across the three t3568-r5-bup cases. Follow-up: T-3867 (BETA.INV's search on the n-term branch).
