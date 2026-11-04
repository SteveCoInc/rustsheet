---
id: T-3719
title: "Incomplete beta: Excel's bgrat in the a0 < 1e-15 band at 0.04 <= y < 0.25 (split from T-3568)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The a0 < 1e-15 cells at 0.04 <= y < 0.25, where Excel sits 1e-14 to 3.8e-13 relative above the true value, have a
model with hold-out counts, or the recorded negatives.

## Context
- Split from T-3568, which did not attempt this item (docs/research/t2464/r5-bup/RESULTS.md section 5). Origin:
  docs/research/t2464/r2-bpser-small/RESULTS.md finding 5. For a0 < 1e-15 (swapped cells, x > 0.5) bratio falls through
  to bup + bgrat. In 0.04 <= y < 0.25 every model and CR miss Excel (`tiny_noapser.out`). These cells have bgrat's
  z = -nu ln x >= 1.1, where grat1 uses its continued fraction, so this is the incomplete-gamma ratio problem.
- Related: D to the bit at b != 1/2 (T-3383; r5-bgrat-fdf1 negatives) and the T-3568 n >= 2 bup composition.
  Method: docs/research/excel-kernel-playbook.md, with a new `r6-*` directory under docs/research/t2464.
- Rules: positive controls first, a real hold-out, record negatives (decision 0056), 15-digit literal truncation.

## Acceptance criteria
- [ ] A RESULTS.md records a model with exact, within-4-ulp and within-15-digit counts on a held-out probe, or the
      negatives tried with their counts.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's T-2464 entries are updated with what is found.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Deferred by decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) parks this task until Phase 1 exits. It is work on the incomplete-beta family whose cells are first wrong only at the 14th or 15th significant digit, or a probe or port that serves only that work. In those regions Excel itself is not correctly rounded (on a sample of 2,026 visibly-off cells, Excel was correctly rounded on 1%, with a median error of 48 to 330 ulp). Matching those digits means reproducing Excel's rounding noise. Decision 0267 accepts those cells. Reopen the task only if a cell it owns is first wrong at the 13th digit or earlier.
