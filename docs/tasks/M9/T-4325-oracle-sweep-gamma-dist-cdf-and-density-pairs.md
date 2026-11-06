---
id: T-4325
title: "Oracle sweep: GAMMA.DIST cdf and density pairs inside the large-shape window from 2a/3 to 0.8a, to place where the cdf leaves the density's error"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph-filled probe under probes/ holds GAMMA.DIST density and cdf pairs (and the upper tail) at the same
exact x inside the large-shape window, from x/a = 2/3 to 0.8, at shapes where one ulp of the exponent spans many
result ulps. With it, T-4326 can read where Excel's cdf stops sharing the density's exponent error and fit that cut
with a hold-out, instead of guessing from the 55 cached in-window cdf cells.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 062b67cf8). `make unowned-off ALL=1` lists one visibly-off cell
  with no live owner: probes/t3252-gamma-prefactor-window Calc!A44 `GAMMA.DIST(35,50,1,TRUE)`, Excel
  0.009845502476408533, ours 0.009845502476408544, 50-digit value 0.0098455024764085416 (Excel is -8.7E-16 relative,
  about 5 ulp low; ours 1.4 ulp high). T-4326 owns that cell and needs this sweep first ("Needs an oracle sweep",
  gap-triager rule).
- What is known (docs/excel-semantics/statistics.md, "Incomplete gamma: accurate kernel", the T-4255, T-4290 and
  T-4296 paragraphs): inside the window (|x - a| <= 0.2 (x + a)) Excel's density carries a per-unit exponent error
  of about +-0.5 ulp(ln u) that no tested form reproduces (T-4308 owns reading it). The cdf carries the same error up
  to about 0.70a and decouples from about 0.72a: t3252 holdout A23 and A37 (densities at shapes 100 and 500) carry
  -0.65 ulp per unit, their cdfs A24 and A38 only -0.17 and -0.13. The engine's P and Q keep `gamma_prefactor`'s chain
  (crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs), not the density's `mode_density`.
- Why the cache is not enough: the 55 cached in-window cdf cells between 2a/3 and 0.8a (t3252-gamma-prefactor-holdout
  15, -holdout-2 12, -sweep 16, -window 3, t4255-gamma-window-exact-ratio G5:G13) are at shapes 30 to 500, where one
  ulp of the exponent is a fraction of a cdf ulp, so the per-unit error does not read out. T-4290 read the density's
  error only at power-of-two shapes 256 to 4096 (probes/t4290-gamma-window-large-pow2, densities only, no cdf).
- Layout (fixed here, before any scoring, so the hold-out is never tuned on):
  - fit columns: shapes 4096, 1024 and 256 at x = a k/1024, k = 683 to 819 (2/3 < x/a <= 0.8), with
    `GAMMA.DIST(x,a,1,FALSE)`, `GAMMA.DIST(x,a,1,TRUE)` and `CHISQ.DIST.RT(2x,2a)` (Q(a, x), exact doubling) for each
    point;
  - hold-out columns: shape 2048 at k + 1, the same three formulas;
  - a power-of-two shape keeps x, a - x and the ratio x/a exact (the T-4290 rationale).
  About 140 points per shape, 2,000 cells or fewer in total.
- Probe freeze (decision 0267 item 5): probes-numeric takes no new case here, so the case goes under
  eval/suites/probes/cases/. Write it with a committed generator, docs/research/t4325/make_probe.py, on the pattern of
  docs/research/t4290/make_probe.py (`sweep_case.write_checked_case`), and fill it with
  `cd tools && uv run rustsheet-oracle fill` (excel-graph, canary-verified).
- Attach a 50-digit math sidecar (`.math.json`, mpmath, as t3252-gamma-prefactor-window has) so T-4326 can give each
  Excel cell its per-unit error and run the Gate A envelope check.

## Acceptance criteria
- [ ] eval/suites/probes/cases/t4325-gamma-window-cdf-pairs.case.json exists with the layout above, written by
      docs/research/t4325/make_probe.py, and every cell is oracle-filled from excel-graph (no pending cell).
- [ ] Its `.math.json` sidecar gives every cell to at least 40 significant digits.
- [ ] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel" names the case as evidence (date,
      source), and states that the hold-out is shape 2048 at k + 1, fixed before scoring. No rule or fit is claimed.
- [ ] `make eval-baseline` records the new case's failing cells (a new case, so no regression). The
      visible-parity lock line is the one the tool writes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scoring any form, and any engine change (T-4326, T-4308).
- Shapes below 20 (T-3219), and x/a above 0.8 or below 2/3.
