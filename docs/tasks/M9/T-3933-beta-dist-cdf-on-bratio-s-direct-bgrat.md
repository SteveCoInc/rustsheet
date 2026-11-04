---
id: T-3933
title: "BETA.DIST cdf on bratio's direct bgrat branch at 0 < a0 < 1, a0 != 1/2: take bgrat_increment (2,168 visibly-off cells of probes-numeric t2512-r3-bgrat-split)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
defer: phase-1b
---

## Goal
BETA.DIST's cdf on bratio's direct bgrat branch (bratio's swapped call bgrat(b0, a0, y0, x0, 0) with 0 < a0 < 1,
a0 != 1/2, b0 > 15) is computed by the accurate double-double bgrat the engine already has, not by today's kernel, so
most of probes-numeric t2512-r3-bgrat-split's 2,908 cells come within 15 digits of Excel.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (incomplete beta, bpser) measured on t2464-r2-bpser-small-gl, 0 of 5843 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
- Gap triage 2026-11-03 b. `make eval` on origin/main 07e0c0a8c: probes-numeric t2512-r3-bgrat-split (2,908 BETA.DIST
  cells, excel-graph) passes 35; 2,873 fail (2,524 NumericClose, 349 NumericUlp) and 2,168 are visibly off at the 15th
  digit. `r1c_left_out` classes all 2,168 `phase1b` (both values within 1e-9 of the math), so they leave the M9 exit
  count, but they are the largest visibly-off BETA.DIST group no open task owns:
  - Owned: every visibly-off cell of probes-numeric t2512-r3-bgrat-split (2,168 of its column Calc!A1:A2908; the
    eval's per-case cell list names them). No open task cites the case.
  - e.g. A1 `BETA.DIST(0.8117,30.699999999999,0.7,TRUE)`: Excel 0.0007235467872156648, ours 0.0007235467872156635
    (12 units of 2^-53); A10 `BETA.DIST(0.8117,(128765132*2^-22+53686847*2^-48),0.7,TRUE)`: Excel
    0.0007235467872156447, ours 0.0007235467872156436.
- Why: `bratio/mod.rs` `ported_tails_with` takes `Branch::Bgrat` only at a0 = 1/2 (`bgrat_half`). Its comment says
  "only b = 1/2 is ported (T-2509; T-3582 owns the rest)", but T-3582 is done with negatives on Excel's own noise in D and
  filed no engine task, so every a0 != 1/2 point falls through to `incomplete_beta::beta_reg_tails`.
- The kernel exists: `bratio/bgrat.rs` `bgrat_increment(a, b, x, y)` is TOMS bgrat at 0 < b < 1 in double-double
  (Legendre's fraction for J_0, stored lnx and z as Excel stores them), used today only as D inside L140 bup+bgrat
  (T-3382, `bup.rs`). Reuse it; do not add a second bgrat.
- Evidence of the gain (docs/research/t2464/r3-bgrat/RESULTS.md finding 3, r6-bgrat-q/RESULTS.md table 1, both on these
  2,908 split points): the exact increment scores 191 / 1,587 / 2,069 (bit-exact / within 4 ulp / within 15 digits)
  against Excel; r3-bgrat's x87 grat1 at eps 1e-15 206 / 1,672 / 2,158; TOMS at its own 15e-15 13 / 142 / 162.
  Excel sits a median 4 units of 2^-53 below the exact value, with white noise of sd 5.7-8.5 units that no model
  explains (T-3567, T-3582 negatives), so the accurate kernel is the target (decision 0114 `accuracy`), not a fit.
- Callers: the same `half_bgrat` callers as the L140 port (BETA.DIST's cdf, Student t). F, BINOM and NEGBINOM stay on
  today's kernel here (T-3580, T-3712 own their routing). BETA.INV's search crossing this branch: check, and leave it on
  today's kernel if it moves gated cells (T-3867 owns the search on L140).
- Related open tasks: T-3766 (folds `large_shape_beta.rs`'s double Legendre fraction onto this core), T-3224
  (GAMMA.DIST's incomplete gamma), T-3871 and T-3712 (L140 bup+bgrat). None lists these cells.

## Visible-parity stage
Decision 0083 / 0114 (`accuracy`): the 2,168 cells above are visibly off and the 50-digit reference shows Excel within a
few units of the correctly rounded value while the engine is 10+ units off. Target: the visibly-off count on
t2512-r3-bgrat-split falls to at most 850 (the exact increment's 2,908 - 2,069 = 839, plus a margin of 11), with every
remaining visibly-off cell nearer the 50-digit value (t2512-r3-bgrat-split.math.json) than today's. Cells still off
then go to a follow-up without the key, with `defer: phase-1b` if they are within 15 digits of the math.

## Acceptance criteria
- [x] `ported_tails_with` takes `Branch::Bgrat` for 0 < a0 < 1 through `bgrat_increment` (BETA.DIST's cdf callers);
      `None` (z below the fraction's range, z past `MAX_Z`) keeps today's kernel. The stale "T-3582 owns the rest"
      comment names this task's result instead.
- [x] probes-numeric t2512-r3-bgrat-split: visibly-off cells drop from 2,168 to at most 850; within-15-digit count
      reported in the Outcome next to r6-bgrat-q's exact-Q row (191 / 1,587 / 2,069).
- [x] No gated suite gains a known failure: `make eval-check` reports 0 regressions in every eval/baseline/<suite>.txt,
      and any fixed lines are recorded with `make eval-baseline`.
- [x] Every other probes-numeric BETA.DIST / T.DIST case on the branch (t2464-*, t3382-l140-sweep, t3567-r5-fdf1-hold,
      t3582-r6-gamma-runs) is checked before/after with `rustsheet-eval show --all`; the Outcome lists any cell that
      leaves 15 digits, and none may become further from the math than before.
- [x] No constant is chosen by scoring (no `FITTED` tag). If a choice between forms is ranked on Excel cells (for
      example, x87 grat1 at eps 1e-15 vs the double-double fraction), the decision 0188 R4 guards of
      reviewer-excel-fidelity check 7 apply: hold-out counts on T-2512's md5-odd split, a fresh sweep after the freeze,
      the Gate A envelope result on it, the reproducibility floor, and no keep-matched branch.
- [x] statistics.md "Incomplete beta: the integrated model" states the engine's direct-bgrat path; a unit test pins A1.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Modelling Excel's noise in D (T-3567, T-3582 negatives).
- F, BINOM and NEGBINOM callers (T-3580, T-3712); BETA.INV's search on L140 (T-3867).

## Landing notes
Parked, not landed (2026-11-04). The work below was finished and passed `make check-fast`, design and security review
(PR #3177, head 2a780f616), but decision 0267 (owner ruling, merged to main while this branch was in flight) cuts
T-3933 as Tier D (`defer: phase-1b`): its cells are first wrong only at the 14th or 15th digit. Main's copy of this
file carries the deferral text; merging main here conflicts only in this file. The PR stays a draft. Reopen only per
decision 0267 (a cell this task owns first wrong at the 13th digit or earlier); the change and its A/B are then ready,
needing only a merge of main and the Excel-fidelity review, which had not reported when the work was parked.
Results as measured:
- `bratio/mod.rs` `ported_tails_with` takes `Branch::Bgrat` at every 0 < a0 < 1: Excel's bgrat at 1/2 as before, and
  `bgrat.rs` `bgrat_increment` (the double-double bgrat L140 bup+bgrat already uses for D) elsewhere, with the other
  tail (0.5 - w1) + 0.5; `None` keeps today's kernel. The stale "T-3582 owns the rest" comment now names this task.
  Unit test `direct_bgrat_takes_the_accurate_increment` pins t2512 A1 at the correctly rounded 0.0007235467872156652.
  No second bgrat, no constant chosen by scoring (no `FITTED` tag; the kernel is the accurate value, decision 0114).
- probes-numeric t2512-r3-bgrat-split, by the eval's visibly-off rule (decisions 0083, 0088): visibly off 2,168 to
  823 (target <= 850); exact / within 4 ulp / within 15 digits 35/384/740 to 216/1,605/2,085, against r6-bgrat-q's
  exact-Q row 191/1,587/2,069. The earlier landing-notes count (995) used plain 15-digit equality, which counts cells
  within 4 ulp as off; the eval does not.
- Clean A/B on one base (the change reverted for "before", both binaries built from this branch;
  docs/research/t3933/ab.out, `ab.py` and `ab_visible.py`): only BETA.DIST cells move, all in probes-numeric.
  - t3568-r5-bup-{fresh,hold,tune}: visibly off 873/1,837/1,858 to 668/1,387/1,439; 212 cells newly pass. 163 cells
    move further from both Excel and the 50-digit math at tiny tails (I below about 1E-20): the stored
    z = fl(-ν·ln x), which Excel's D tracks (statistics.md, T-2512 `zstore`), moves I by about z·2^-53 relative with z
    in the hundreds, and Excel itself is that far off the math there (fresh B218: Excel 4.8E-14 relative off).
    1,285 move nearer Excel. These cases are not in this task's no-further-from-math list.
  - t2464-r2-bratio-x29 (listed): 155 cells change and all 155 now pass (bit-exact 2,696 to 2,851 of 3,080). 154 of
    them land on Excel's value 1 ulp from the math through TOMS's (0.5 - w1) + 0.5 complement: Excel is the spec
    (directive 1), so this is the intended move, not a loss. No cell leaves 15 digits.
  - t2464-* others, t3382-l140-sweep, t3567-r5-fdf1-hold, t3582-r6-gamma-runs: no cell changes.
  - m9-beta-dist-a100 and -a50: one cell each, nearer both Excel and the math.
- BETA.INV's search crosses the new branch; no gated cell moves (`make eval-check`: 0 fixed, 0 regressions in every
  eval/baseline suite), so it stays on it.
- statistics.md "Incomplete beta: the integrated model" gains the engine's direct-bgrat bullet.
- The direct branch reaches b0 far past L140's 1E7 cap: at b0 = 1E164 (eval/accuracy BETA.DIST s1 A192) the increment's
  v = 0.25/nu^2 overflowed in double-double to NaN, failing the accuracy gate. `bgrat_dd` now divides by nu twice
  (v underflows to 0, as TOMS's does) and hands a non-finite result back to the caller's kernel; unit test
  `direct_bgrat_at_a_huge_b0_is_finite` pins A192 at Excel's 1. No other cell moves (t2512, t3568-hold, t3382 recounted).
