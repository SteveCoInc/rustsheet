---
id: T-4865
title: "BETA.INV at a tiny shape: when Excel's AS 109 search returns 1 - 2^-53, 2^-53, 1 or a subnormal instead of #NUM!"
milestone: M9
lane: functions-stat
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.INV and BETAINV return Excel's value or Excel's `#NUM!` where the quantile is at an end of the doubles in
(0, 1) at a tiny shape: rounding to 1 at p of 1/2 or less, underflowing at p above 1/2, or with a subnormal flipped
tail at p above 1/2. Excel answers an end value at some of these points and `#NUM!` at others, and no rule that
separates them is known yet.

## Context
- Follow-up of T-3351 (PR #3911, superseding #2573). That PR made a quantile that rounds to 1 at p above 1/2 equal to
  1 - y where the flipped tail's y is a normal double, and left the rest `#NUM!` as before. statistics.md "BETA.INV
  where the quantile rounds to 1 (T-3351)" records the evidence below.
- The three classes, Excel against ours (`#NUM!` in every one):
  1. p of 1/2 or less, quantile rounds to 1: Excel 1 - 2^-53 at 13 Gate A points (accuracy/BETA.INV/s1 and
     BETAINV/s1 lines in eval/accuracy/known-failures.tsv naming this task) and at probes/t3351-beta-inv-p-near-1
     D197-D202, D204, D210. But Excel is `#NUM!` (ours too, right today) at s1 BETA.INV A89, A92, A95, A109 and
     BETAINV A83, A91 (b from 3.8E-15 to 1.6E-8), and at the tiny-p points A184, A189, A195, A199. Neither b, p, a nor
     the distance of the root from 1 separates the two sets.
  2. p above 1/2, quantile underflows: Excel 2^-53 at s1 BETA.INV A88, A97, A108 and BETAINV A85, A88, A89, A99,
     A105, A108 (alpha 1.5E-15 to 6E-7), but `#NUM!` (ours too) at s1 BETA.INV A94, A105 and BETAINV A82.
  3. p above 1/2, quantile rounds to 1 and the flipped tail's y is 0 or subnormal: Excel `#NUM!` at most such points
     and 1 at 8: probe A199, A209-A211, C199 and probes-numeric/t3351-beta-inv-sweep A88, A89, A122 (y down to
     1E-700). The lower-tail mirror: probe B199, B209-B211 are Excel 0 (a subnormal quantile shown as 0), ours
     `#NUM!`; sweep A175 is Excel `#NUM!` where ours is 1.5E-112, and A188 Excel 1.4E-153 where ours is `#NUM!`.
- T-3351 tried two things (its "Blocked because" section). (a) Applying the flip to every end (1 - 2^-53 for class 1,
  2^-53 for class 2) moved about 60 cells to Excel but 14 Gate A points away from Excel's `#NUM!`. (b) A simulation
  of AS 109 as revised (the statrs port's start, step control and `acu`) on the ported BETA.DIST kernel ended at
  1 - 2^-53 or 2^-53 at most of Excel's `#NUM!` points. The outcome looks decided by the exact rounding of Excel's
  Newton steps next to 0 and 1, so the likely route is Excel's own kernel values at those x (probe `BETA.DIST` at
  x = 1 - k 2^-53 and k 2^-53 for the shapes above) before the search.
- probes-numeric/t3566-beta-inv-tiny-shape-and-range shows the same classes; its wrong cells were accepted under
  decision 0267 on T-3351 and are not owned here.
- Unverified branch: at a shape above 1E5, `upper_quantile_near_one` takes T-1538's `beta_lower_crossing` for y. No
  evidenced cell reaches it (with beta above 1E5 the quantile is near 0, so only alpha above 1E5 with a small beta
  does). Probe points to add under eval/suites/probes/ (not probes-numeric): `BETA.INV(1-1E-12,2E5,1E-3)` and
  `BETA.INV(1-1E-8,1E6,0.01)`.
- Hand-derived unit-test row `BETA.INV(1-1E-9,10,0.03,3,5)` = 5 (test_fn_beta_dist.rs) needs an oracle cell.
- Code: crates/rustsheet-engine/src/functions/statistical/beta.rs (`beta_inv_of`, `upper_quantile_near_one`,
  `lower_quantile_underflows`).

## Acceptance criteria
- [x] Every probe, sweep and Gate A cell listed in the three classes above has Excel's value within 15 digits or
      Excel's error, or the task records the rule that would decide it as not found and blocks as time-boxed
      (decision 0056). Not found; blocked (Blocked because).
- [x] No cell that matches Excel today moves away from it (the class 1 and 2 `#NUM!` points above included). No
      engine behavior changed.
- [x] Any cut chosen by scoring candidates against cells is a fit: decision 0188 R4 guards (hold-out split fixed
      before scoring, a fresh oracle sweep after the freeze commit, `FITTED` tag, `scripts/check_fitted.py`). No cut
      chosen.
- [x] The shape-above-1E5 probe points are filled by the oracle and match.
- [ ] The rule is in docs/excel-semantics/statistics.md; the Gate A lines that pass are removed. The findings are
      there; no rule and no Gate A line passes yet.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A DBL_MIN shape (T-4663) and a huge shape's `#NUM!` (T-3862, T-4713).
- The upper-tail search at a tiny alpha and a large beta (T-4866).

## Blocked because
Blocked because: time-boxed (decision 0056)
- Candidates tried (both on the class 1 and 2 cells of accuracy/BETA.INV/s1 and BETAINV/s1: 21 Excel end values, 9
  Excel `#NUM!`s):
  1. Excel's kernel next to the end (the route this task named). Excel's BETA.DIST at every iterate of an AS 109 replay
     (AS R19's acu, the published start clamp to [1E-4, 0.9999], 50-digit cdf and ln B; 1,154 cells,
     docs/research/t4865/kernel_probe, excel-graph 2026-11-08) is within 3.8E-15 relative of the 50-digit cdf, and the
     replay on Excel's own values takes the same path and ends at 1 - 2^-53 or 2^-53 at all 30 cells: 21/30 agree with
     Excel, the 9 `#NUM!`s missed. The kernel does not decide them.
  2. A step or division budget in that replay. Total step divisions (g /= 3) are at most 714 at every Excel end value
     and at least 810 at every Excel `#NUM!` (30/30 on this set, a cut found by looking, so a fit with no hold-out), but
     replays of Excel's normal tiny quantiles reach 38,002 divisions (s1 A82, Excel 5.3E-214) and 7,596 iterations, and
     at the subnormal-y class (probe A1-A214, sweep) Excel's 1s and `#NUM!`s overlap in iterations, divisions and the
     most divisions in one step (docs/research/t4865/replay_all.txt). Not adopted: no budget fits all the evidence.
     The most divisions in one step also overlaps (end values up to 31, `#NUM!` down to 15).
- What would reopen it: Excel's iterates themselves (not reachable from a cell), a different AS 109 variant whose
  trajectory reproduces T-2555's tiny-quantile early stops as well (those cells are also on no AS 109 path from this
  start, statistics.md "BETA.INV at a huge shape is AS 109's Newton search (T-1538)"), or an owner ruling that accepts
  the classes under decision 0267.

## Outcome
Research only; no engine behavior changes. Eval delta: +8 passing cells (new case
probes/t4865-beta-inv-huge-alpha-near-one, 8/8), no regressions.
- New probe probes/t4865-beta-inv-huge-alpha-near-one (8 cells, excel-graph): the shape-above-1E5 branch of
  `upper_quantile_near_one` (`BETA.INV(1-1E-12,2E5,0.1)` = 1; `#NUM!` where y underflows, both forms) and the
  hand-derived bounds row `BETA.INV(1-1E-9,10,0.03,3,5)` = 5 (A5), all matching. beta.rs's TODO(T-4865) and the
  unit test's "needs oracle confirmation" note now cite it.
- docs/research/t4865: the replay (iteration_counts.py, replay_all.py and its output), the kernel probe generator, case
  and Excel fill (gen_kernel_probe.py, kernel_probe.*), and the replay on Excel's values (replay_excel_kernel.py).
- statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)" records both findings and marks the shape above
  1E5 VERIFIED.

