---
id: T-4409
title: "Gamma upper tail below 1.3a at integer and half-integer shapes, and ours near x = a: find the shared kernel's step (T-4368 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Two outcomes:
- Below 1.3a, Excel's gamma upper tail Q(a, x) is reproduced for both integer and half-integer shapes. Integer shapes
  reach it through POISSON.DIST and CHISQ.DIST.RT; half-integer shapes through CHISQ.DIST.RT with odd df. If it cannot
  be reproduced, the step that carries Excel's error is located.
- Ours is within 1 ulp near x = a, where Excel is.

## Context
- T-4368 (probes/t4368-poisson-shape-and-edge, docs/research/t4368/score.py; statistics.md, T-4368 paragraph) found:
  - POISSON.DIST(k, m, TRUE) equals CHISQ.DIST.RT(2m, 2k + 2) bit for bit at all 94 points.
  - At half-integer shapes, CHISQ.DIST.RT(2m, 2a ± 1) is as far from the truth as the integer-shape cdf, up to 41 ulp.
    So Excel's error lies in a gamma-Q kernel shared by all shapes, not in an integer-shape path.
  - With m stepped across 1.0a (rows 43:94, m/a 0.996 to 1.008), Excel is within 1 ulp in every column. So no
    series/fraction switch at x = a shows, and the error grows somewhere between 1.008a and 1.3a.
- Next probe: a ladder in m/a from 1.01 to 1.3 at fixed k (50, 100, 300), at integer and half-integer shapes, to find
  where the error starts and whether it jumps there. Probes go in the `probes` suite (decision 0267 item 5). Score with
  a hold-out fixed before scoring (decision 0188 R4).
- Ours near x = a: our CHISQ.DIST.RT at half-integer and integer df, and our GAMMA.DIST at shape a + 0.5, are several
  ulp off where Excel is within 1 ulp (B72, D73, C83, E91, E94 ...). That is our own accuracy gap, not an Excel
  quirk. Code: crates/rustsheet-engine/src/functions/statistical/ (the gamma kernels, excel_tail.rs).

The visibly-off cells this task owns:
- probes/t4368-poisson-shape-and-edge: A12, A13, A14, A34, A39; B8, B13, B14, B16, B20, B37, B39, B40, B64, B72,
  B76, B94; C8, C11, C12, C13, C15, C16, C18, C31, C41, C42, C46, C52, C55, C83; D7, D8, D13, D15, D73, D91, D94;
  E91, E94.
- Taken over from T-4368: probes-numeric/t4322-poisson-window-r4-fresh G3, B8, B9, A10 and C11;
  probes/t4291-poisson-far-above-mean D2 and H2; probes/t4302-poisson-cdf-beside-pmf B21 and E19; any visibly-off
  cell of probes-numeric/t4357-poisson-companions column A.

## Acceptance criteria
- [ ] The ladder probe is committed with oracle values. The scored models are recorded in statistics.md, with the
      hold-out fixed before scoring. A model that matches Excel on the hold-out and on a fresh sweep is implemented.
- [ ] Ours is within 1 ulp of Excel on rows 43:94 of probes/t4368-poisson-shape-and-edge. If a cell is not, it is
      listed with the reason.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed above are visibly off, so this task is in the campaign.

## Out of scope
- CHISQ.DIST.RT above 1.3a (T-4346, T-4320).
