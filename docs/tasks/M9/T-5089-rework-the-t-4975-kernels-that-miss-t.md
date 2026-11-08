---
id: T-5089
title: "Rework the T-4975 kernels that miss T-4974's validation probes (131 cells) and decision 0351's accepted regressions"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The T-4975 kernels pass main's T-4974 validation probes they now miss (131 cells) and decision 0351's accepted regressions shrink.

## Context
Decision 0351 lists them: probes/t4974-* (beta-inv-as109-accuracy-exponent 42, beta-inv-large-shape-round-trip 28, f-inv-back-transform 17, f-inv-as109-tolerance-cap 8, chisq-inv-unit-half-boundary 9, gamma-split-complement-region 5, t-dist-df1-and-large-df-centre 5, lognorm-density-holdout 4, others), the LINEST/GROWTH QR cells (14 at 260–1022 ulp in t4590/t4765), and the AS 109 centre cells such as `T.INV(0.2,5)`. Also the CHISQ right-tail P search above 0.9999 keeps Newton to hold one gated cell (a keep-matched branch to remove).

Filed by T-4975's review (PR #4073).

## Acceptance criteria
- [ ] The goal holds, shown by tests or eval cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
