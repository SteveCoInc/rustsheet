---
id: T-4015
title: "FDIST and F.DIST on BupBgrat at a0 = 1/2: gen-m9 b6593d7dd13f2dac A50 and 3a25a03eb94fcfbd A181, the Gate B face of deferred T-3472 (accuracy)"
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
The two gen-m9 F cells below are within half a unit of the 15th significant digit of Excel's value. Both are on
bratio's `BupBgrat` branch at a0 = 1/2. No gen-m9, ironcalc-calc or probes-numeric cell that is within 15 digits today
moves off.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 30a36f59, excel-graph). `make phase1-exit` reports "owners not
  met: 109 visibly-off cells with no open owner (109 owned only by a deferred task)". `make unowned-off ALL=1` lists
  these two cells among them.
  - Gap triage 2026-11-03 moved them from deferred T-2221 to T-3472. T-3472's own run (#3059, 2026-11-04) then
    deferred T-3472 to Phase 1b, as last-digit work inside Gate A's envelope.
  - gen-m9 is a Gate B suite, and decision 0211 counts every visibly-off cell there whatever its math. So a deferred
    owner leaves FDIST below Gate B and the `owners` line unmet. FDIST has 3 visibly-off cells of 227 rate cells:
    A50 is one, and T-4016 and T-4018 own the other two.
  - Precedents: T-3952 and T-3954, which are Gate B faces of deferred tasks.
  - This task takes the two cells. T-3472 and T-2221 keep only probe cells and bit-level residue, and both name this
    task.
- The cells (`make eval-show CASE=gen-m9/<case>.case.json`, NumericClose). References are 50-digit mpmath 1.3
  `betainc` at the double arguments: F as I_x(d1/2, d2/2), the right tail as I_v(d2/2, d1/2). Relative errors are
  given as Excel / ours.
  - gen-m9/b6593d7dd13f2dac Calc!A50 `=FDIST(Data!$E6,5,1.5)`, which is `FDIST(1.005,5,1)` because Excel truncates the
    df. Excel 0.6356868721336002, ours 0.6356868721336008, math 0.63568687213360044627: -3.9E-16 / +4.9E-16. The
    correctly rounded value, 0.6356868721336004, is within 15 digits of Excel.
  - gen-m9/3a25a03eb94fcfbd Calc!A181 `=F.DIST(Data!$B9,MONTH(17720),1,TRUE)`, which is `F.DIST(1,7,1,TRUE)`. Excel
    0.3506166628202075, ours 0.35061666282020815, math 0.35061666282020752657: -1.3E-16 / +1.77E-15. The correctly
    rounded value, 0.35061666282020754, is within 15 digits of Excel. A181 is the complement of probes-numeric
    `F.DIST.RT(1,7,1)` on T-2220's right-tail sweep (a T-3472 cell, deferred), so one fix should land both.
- Root cause, from T-3472's measurement in its "Progress notes":
  - Both cells are `BupBgrat`, swapped to a0 = 1/2: A50 at b0 2.5, x0 0.166; A181 at b0 3.5, x0 0.125.
  - F does not port bgrat at a0 = 1/2 (`half_bgrat` is false in `fisher.rs`), so both go to Cephes
    (`beta_reg_tails`).
  - Routing F through bup + `bgrat_half` (the BETA.DIST path) lands A50 and A181 exactly, but the net is even:
    - On gen-m9, 6 visibly-off cells come within 15 digits and 6 go off. Two of those are bit-exact today:
      gen-m9 7c20351562a2d71d A205 and c18dbf21f11093f3 A151, both `FDIST(x,1,9)`.
    - On probes-numeric, 22 come within 15 digits and 21 go off.
  - On the d2 = 1 sweep points, mpmath puts Excel itself 5 to 12 ulp off. Excel's bgrat there is neither ours nor the
    math.
- The work, then:
  - Find what separates the points where Excel's a0 = 1/2 bgrat is accurate (these two) from the points where it is
    not (the `FDIST(x,1,9)` cells and T-3474's d2 = 1 cells). Candidates are the bup term count n, x0 and b0.
  - Take the accurate kernel only where Excel is accurate. Derive that domain from the structure of TOMS 708's
    dispatch, not from the set of cells that match today.
  - If no structural rule separates the two sets: record the measured split in statistics.md, "Incomplete beta is
    TOMS 708 `bratio`". Then set the task to blocked, with a `## Blocked because` section that names the evidence
    needed (a sweep over b0 and x0 at a0 = 1/2).
- Evidence on file: the a0 = 1/2 points of T-2220's five probes-numeric F sweeps (cdf, pdf, rt, d1-1, d2-1) and of
  T-3567's df1 = 1 hold-out, and gen-m9's FDIST and F.DIST cells. (Cases are named here by their tasks, so this task
  does not claim their cells for `exit_owners.py`.)
- Code: crates/rustsheet-engine/src/functions/statistical/fisher.rs (`f_beta_arguments`, `half_bgrat`); `bratio/`
  (`dispatch`, `Branch::BupBgrat`, `bgrat_half`); `incomplete_beta.rs` (`beta_reg_tails`).

Cells this task owns (split with T-3472 and T-2221, which keep only the bit-level residue):
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A50
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A181

Correction (gap triage 2026-11-04c, `make eval` on origin/main 9a01e9b25): the two `FDIST(x,1,9)` cells are not
bit-exact. Both are NumericUlp known-failure lines of eval/baseline/gen-m9.txt, and no open task named them in a form
`exit_owners.py` reads. They are `FDIST(2,1,9)`: Excel 0.190947117169724, ours 0.19094711716972404, mpmath 50-digit
I_{9/11}(4.5, 0.5) = 0.19094711716972392844, so Excel is +3.7E-16 and ours +5.8E-16 relative. They are within 15
digits and are not stage cells. This task owns them as the a0 = 1/2 counter-case its rule must explain:
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A205
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A151

## Visible-parity stage
`campaign: accuracy` (decisions 0083, 0114). Excel is 1.3E-16 to 3.9E-16 from the math; ours is 4.9E-16 to 1.8E-15
off. The stage is met when:
- A50 and A181 are within half a unit of the 15th significant digit of Excel's value;
- no gen-m9, ironcalc-calc or probes-numeric cell that is within 15 digits today goes visibly off.

Bit-level criteria still unmet then go back to T-3472 (`defer: phase-1b`).

## Acceptance criteria
- [ ] gen-m9/b6593d7dd13f2dac Calc!A50 and gen-m9/3a25a03eb94fcfbd Calc!A181 leave eval/baseline/gen-m9.txt or are
      within 15 digits of Excel. The Outcome shows the `make eval-show` output for both and names the branch each
      takes.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions). In
      particular, gen-m9/7c20351562a2d71d Calc!A205 and gen-m9/c18dbf21f11093f3 Calc!A151 move no further from
      Excel's value than today's 0.19094711716972404 (they leave the baseline if they become bit-exact).
- [ ] If the domain of the accurate a0 = 1/2 kernel is chosen by scoring candidates against oracle cells, it is a fit
      (reviewer-excel-fidelity check 7, decision 0188 R4). The Outcome then shows every R4 guard:
      - Hold-out: counts (bit-exact, within 4 ulp, within 15 digits) beside the correctly rounded candidate, on a
        split stated before scoring.
      - Fresh sweep: a log-uniform sweep at a0 = 1/2, oracle-filled after the rule is frozen, with its Gate A
        envelope result and every point outside the envelope listed.
      - Tag: a `FITTED(T-4015, n_cells, range)` tag and range on the fitted piece, with accurate math outside it.
      - Floor: nothing fitted below the reproducibility floor (4 ulp web against desktop, decision 0112).
      - No keep-matched branch: no domain defined by the cells that match today.
- [ ] Unit tests pin A50's and A181's values at Excel's, naming the case and cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3472's six probe cells, and every cell where Excel itself
  is off the math (T-3474, T-3592; Phase 1b).
- The L140 bup+bpser and BpserLarge gen-m9 cells (T-4016), and FDIST at a huge df2 (T-4018).

## Deferred by decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) parks this task until Phase 1 exits. It is work on the incomplete-beta family whose cells are first wrong only at the 14th or 15th significant digit, or a probe or port that serves only that work. In those regions Excel itself is not correctly rounded (on a sample of 2,026 visibly-off cells, Excel was correctly rounded on 1%, with a median error of 48 to 330 ulp). Matching those digits means reproducing Excel's rounding noise. Decision 0267 accepts those cells. Reopen the task only if a cell it owns is first wrong at the 13th digit or earlier.
