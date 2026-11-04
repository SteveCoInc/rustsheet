---
id: T-3582
title: "Incomplete beta: the white noise in Excel's direct bgrat increment at b != 1/2 (Excel's own Q(b, z) inside bgrat)"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Excel's direct bgrat increment D at b != 1/2 is modelled to the bit, or the source of its noise is named with
evidence.

## Context
- Split from T-3567 (docs/research/t2464/r5-bgrat-fdf1/RESULTS.md section 1 item 4 and section 3). Along the
  split probe's runs of 1,024 consecutive doubles (t2512-r3-bgrat-split, (30.7, 0.7) and (40, 0.03)), Excel's D
  minus the exact I at its stored ln x is white noise of sd 5.7 and 8.5 units of 2^-53. Only the stored
  z = fl(-ν·ln x) shows. The prefactor's form, ln x by log1p and grat1's CF in doubles were all tried and recorded
  as negatives; the best form is still r3-bgrat's stored-z ratio, 246/1,881/2,357 of 2,908.
- Lead: bgrat may take Q(b, z) from Excel's own incomplete gamma (as GAMMA.DIST computes it) instead of TOMS grat1.
  The worksheet cannot show Q at b < 1 directly (T-2512: CHISQ.DIST.RT truncates its df, 1 - GAMMA.DIST loses the
  digits), so probe GAMMA.DIST(z, b, 1, TRUE) runs at small z where P is small and its own noise is visible, and
  compare its noise model with D's on the same (b, z) runs. Work in docs/research/t2464/ in a new `r6-*` directory,
  importing r3-bgrat's `bgld.py` (it now takes an `efn` and a `cfd` Q mode) and r5-bgrat-fdf1's `dreg.py` method.
- Rules: positive controls first; a real hold-out; record negatives (decision 0056); truncate formula literals to 15
  digits as Excel does.

## Acceptance criteria
- [x] A RESULTS.md records a model of D with exact, within-4-ulp and within-15-digit counts on a held-out probe, or
      the negatives tried with their counts.
- [x] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [x] statistics.md's "Incomplete beta: the integrated model" entry is updated with what is found.
- [x] Engine follow-ups are filed for any solved piece.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes; F's arguments (T-3580); the other T-3468 items (T-3568 to T-3572).

## Outcome
- docs/research/t2464/r6-bgrat-q/RESULTS.md: negatives. D's noise is not a different Q(b, z) fraction at bgrat's
  start: NR's gcf by modified Lentz (the bfrac loop of T-3571) and Cephes igamc, in doubles, score 145 and 142
  bit-exact at best of the 2,908 split points against 191 for the exact Q and 206 for r3-bgrat's x87 grat1, and raise
  the run sd (to 8.3-9.1 and 15.5-22.1 from 6.6 and 8.4). r3-bgrat's `bgld.bgrat_ld` now takes a callable Q mode.
- New probe probes-numeric t3582-r6-gamma-runs (4,096 cells, excel-graph 2026-11-02): GAMMA.DIST(z, b, 1, TRUE) on
  runs of 1,024 consecutive doubles. Excel's P is faithfully rounded at b = 0.03 (22-24% of cells 1 ulp off CR, about
  0.2 units of its own error), +2.3 units of 2^-53 high at b = 0.7 near z = 1 (one web fill, inside the web/desktop floor), and noisy (sd 3.7, autocorrelated, not white) at b = 0.7 near z = 0.3. No series (NR gser, Cephes igam,
  the alternating series; doubles or x87) and no stored intermediate explains that noise. So D's b = 0.03 noise
  (sd 8.5) does not come from Excel's lower incomplete gamma at that b, which adds only about 0.2 units; the source stays open.
- statistics.md "Incomplete beta: the integrated model" updated (the T-2512 "Why TOMS bgrat lost" item).
- Engine follow-ups: none solved. The probe's 220 visibly-off engine cells are added to T-3224's owned cells.
- Fits: none (every candidate is a cited algorithm; nothing chosen by scoring is kept).
- Eval delta: none (research and one probe; probes-numeric is gate = false).

