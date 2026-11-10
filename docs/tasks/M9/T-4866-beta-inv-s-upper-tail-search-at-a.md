---
id: T-4866
title: "BETA.INV's upper-tail search at a tiny alpha and a large beta lands far off the root"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
`BETA.INV(1-1.10491974750915E-14,0.0050070029475036,171.171095424379)` returns Excel's 0.12939405629301814 (the
50-digit math agrees), not the engine's 2.925059559426143E-5.

## Context
- Found by T-3351's fresh sweep (probes-numeric/t3351-beta-inv-sweep A200, excel-graph). It is the only point of
  that sweep outside the Gate A envelope that is a number on both sides. The quantile does not round to 1, so
  T-3351's code path (`beta.rs` `upper_quantile_near_one`) never runs there.
- PR #2573's analysis (2026-10-31, on an older `beta.rs`): at these shapes the upper-tail incomplete beta the
  search crosses returns exactly 0 or 1 over a wide band, so the refinement moves statrs' correct start off the
  root. Re-check it against the current search (`beta_quantile`, `bratio::beta_search_tails` with p above 1/2).
- Code: crates/rustsheet-engine/src/functions/statistical/beta.rs `beta_quantile`, bratio/mod.rs.

## Acceptance criteria
- [x] Sweep A200 is within 15 digits of Excel; no other cell of the BETA.INV probes and sweeps moves away from
      Excel (`make eval-check`).
- [x] The cause is recorded in docs/excel-semantics/statistics.md (or the engine note, if it is ours alone).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Quantiles at an end of the doubles (T-4865).

## Outcome
- Cause (ours alone): A200 is on bratio's direct bgrat branch (a0 = 0.005, b0 = 171), which is ported only at
  a0 = 1/2, so the search fell back to the Cephes kernel. Its tails are exactly 1 and 0 from x = 1E-4 to 0.2 there,
  and the crossing stopped at 2.925059559426143E-5. statrs' start was right; PR #2573's reading holds on the
  current search.
- Fix: `bratio::beta_search_tails` takes the direct bgrat (`search_direct_bgrat`, through the existing
  `bgrat_increment`) where Cephes' tails are an exact 0 and 1 on that branch. A200 is now 0.1293940562930182, 2 ulp
  from Excel's 0.12939405629301814 (within 15 digits). BETA.DIST's cdf is unchanged.
- Taking bgrat at every point of the branch was rejected: in the same sweep it moves A143 (Excel's bits to 16 ulp),
  A157, A164, A168, A176 and A178 further from Excel and 3 cells nearer. The guard keys on the kernel's own
  degenerate output, not on Excel's values.
- Before/after `rustsheet-eval show --all` over all 182 BETA.INV/BETAINV cases in eval/: only A200 changes.
  `make eval-check`: 0 regressions, 0 fixed (probes-numeric is not gated). Gate A: 0 failing.
- Unit test `upper_tail_search_crosses_direct_bgrat_at_a_tiny_alpha` pins A200. statistics.md has a new entry,
  "BETA.INV's search at a tiny alpha and a large beta: Cephes' exact 0 and 1 (T-4866)".
- Follow-up: T-5066 (probe BETA.DIST's cdf where Cephes is an exact 0 or 1 on this branch).
