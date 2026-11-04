---
id: T-3766
title: "Fold large_shape_beta's double Legendre fraction and bgrat series onto bratio's double-double bgrat core"
milestone: M9
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
One definition of Legendre's continued fraction for Q(b, z)/r and of bgrat's `d_n J_n` series: the double stand-in in
`large_shape_beta.rs` (`legendre_fraction`, `upper_over_prefactor`, `correction_series`) uses the double-double core in
`bratio/bgrat.rs` (`upper_gamma_fraction`, `asymptotic_sum`), or is removed where the bratio port now covers its points.

## Context
T-3382 (PR #2776, quality review) generalized `bratio/bgrat.rs` to 0 < b < 1 in double-double, which made the
duplication with `large_shape_beta.rs` (T-2528, an f64 accuracy stand-in whose module docs already say the bratio port's
bgrat branches replace or absorb it) visible. The two differ in precision (double vs double-double) and in the fraction's
domain (`large_shape_beta` takes z >= b + 1 and falls back to `incomplete_gamma` below; bgrat.rs takes z >= 1) and stop rule
(bottom-up at doubled depth vs modified Lentz). Callers of `large_shape_lower`: F.DIST's and BETA.DIST's large-shape
paths (probes-numeric t1530-f-df-cut, t2464-r2-bpser-small-tiny).

## Acceptance criteria
- [ ] `large_shape_beta.rs` no longer has its own Legendre fraction or bgrat series (it calls bgrat.rs's, rounded to double,
      or is retired where the port covers its domain).
- [ ] No gated or probes-numeric cell moves from within 15 digits to visibly off (`rustsheet-eval show --all` before and
      after on every case naming F.DIST, FDIST, F.INV, BETA.DIST, BETADIST or BETA.INV); `scripts/check_duplication.py`
      does not go up.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Deferred by decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) parks this task until Phase 1 exits. It is work on the incomplete-beta family whose cells are first wrong only at the 14th or 15th significant digit, or a probe or port that serves only that work. In those regions Excel itself is not correctly rounded (on a sample of 2,026 visibly-off cells, Excel was correctly rounded on 1%, with a median error of 48 to 330 ulp). Matching those digits means reproducing Excel's rounding noise. Decision 0267 accepts those cells. Reopen the task only if a cell it owns is first wrong at the 13th digit or earlier.
