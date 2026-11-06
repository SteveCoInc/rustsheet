---
id: T-4254
title: "Probe BETA.DIST and T.DIST.RT where L140 bup's prefactor crosses DBL_MIN (T-4045's zero rule, evidenced only on F)"
milestone: M9
lane: functions-stat
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for BETA.DIST and T.DIST.RT at the edge of T-4045's zero rule. The rule applies to every caller of
bratio's L140 bup+bgrat branch, but only F.DIST.RT and FDIST are probed.

## Context
- T-4045 (PR #3242): Excel's L140 bup+bgrat tail is exactly 0 when bup's prefactor ret = P·x0·y0/c (P the Beta(c, a0)
  density at y0) is below DBL_MIN. Evidence: probes-numeric/t4045-f-dist-rt-zero-cut, three F families whose brackets
  meet only around ln(DBL_MIN). The engine applies it to F's tails only (`bratio::f_closed_or_ported_tails`).
  BETA.DIST and the t share the branch but are unprobed (reviewer-excel-fidelity, T-4045 round 1). BETA.DIST's cdf
  also feeds F.INV's search at a huge df1: a 0 there moved `F.INV(1.43795578761177E-262,107702,58)` (accuracy/F.INV/s1
  A146) off the math, while Excel's own F.INV there (0.0239) is far from the math (0.0401). So check whether Excel's
  BETA.DIST zero explains Excel's F.INV.
- Probe: `BETA.DIST(x,a,b,TRUE)` with a in the millions and b < 40 (b0 = b after bratio's swap, x0 > 0.7), and
  T.DIST.RT at a huge df where t's kernel takes L140 bup+bgrat (check `bratio::dispatch` first). Walk x so that
  ln(x0^a0·y0^c/B(c, a0)) crosses -708.396 in steps of about 1, at two shapes per function, at both c = 1/2 and c = 1.
  Put it in eval/suites/probes unless decision 0267's probe freeze allows a probes-numeric case for it. Score it once
  against today's engine (this is the post-freeze fresh check of T-4045's rule).

## Acceptance criteria
- [ ] An excel-graph probe case of BETA.DIST and T.DIST.RT cells around the edge, oracle filled.
- [ ] Each cell where Excel and ours disagree on 0 against a number has an open owner task (or this task fixes it), and
      statistics.md "Where the exact 0 starts (T-4045)" records the result for BETA.DIST and the t.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F (T-4045 probed it); the df1 = 2 closed-form zeros.

## Handoff (resumed)
Escalated by a sonnet run without starting the probe (work-loop step 5b: numeric-edge probe design and a post-freeze fresh scoring against T-4045's rule). Oracle check: `rustsheet-oracle status` showed excel-graph OK at claim time. Next: read `bratio::dispatch` to find which BETA.DIST (a in the millions, b < 40, x0 > 0.7) and T.DIST.RT (huge df) cells take the L140 bup+bgrat branch; build the probe walking ln(x0^a0·y0^c/B(c,a0)) across -708.396 in steps of about 1 at two shapes per function and c = 1/2 and 1; check decision 0267's probe freeze for where the case may live; fill with the oracle; score once; then test whether Excel's BETA.DIST zero explains Excel's F.INV(1.43795578761177E-262,107702,58) (accuracy/F.INV/s1 A146). Record in statistics.md "Where the exact 0 starts (T-4045)".

## Blocked because
Blocked because: duplicate of PR #3469 (branch agent/functions-stat/T-4254-beta-t-zero-cut), which a sonnet run claimed
32 seconds after this branch's handoff and finished first, with the same findings and follow-up T-4437. Do not resume
this branch for T-4254. What it holds beyond #3469, to salvage under T-4437 once #3469 merges (opus run, 2026-11-06):
- Engine: the worksheet BETA.DIST cdf asks T-4045's rule (`beta.rs`, `bratio::l140_prefactor_zero_tails` made
  `pub(crate)`), keeping `beta_dist_lower_tail` (F.INV's search) unchanged; unit test
  `l140_prefactor_zero_is_beta_dists_zero`. It makes probes/t4254-beta-t-zero-cut C10:C17 and D10:D17 exact (a0 = 5E6,
  n = 25 and 29, past the n <= 9 port).
- Probes (excel-graph, filled 2026-11-06): probes/t4254-beta-t-zero-cut (130 cells; BETA.DIST at (2E6, 5.5), (2E6, 6),
  (5E6, 25.5), (5E6, 30), T.DIST.RT at df 1E6 and 1E8, and an F.DIST walk at (107702, 58) with F.INV A146) and
  probes/t4254-beta-t-zero-reach (81 cells; the earlier cut against b at fixed x, against a from 3000 to 5E7, and the
  t against df from 1 to 1E8). Both should be renamed to avoid #3469's ids before landing.
- Findings beyond #3469: at a = 2E6 and the same x, b = 2.5 and 3.5 are 0 while b = 5.5 is 8.5E-291 (the earlier cut
  reaches n = 5 too and looks like a cut on w between 2.8E-295 and 8.5E-291); the t's cut moves with df (numbers at
  2.17E-300 for df 5 to 1E6, 0 at df 1E8); Excel's F.DIST at (107702, 58) is 0 at both Excel's F.INV root
  (0.0238674729318850) and the math's (0.040061361917045), so Excel's F.INV root is inside its cdf's flat 0
  (draft follow-up docs/tasks/M9/T-4442-f-inv-at-a-p-inside-excel-s.md). T-4441 here duplicates T-4437 and should be
  dropped, its cell list merged into T-4437.
