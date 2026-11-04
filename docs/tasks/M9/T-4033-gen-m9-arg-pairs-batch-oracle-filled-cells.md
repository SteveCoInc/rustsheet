---
id: T-4033
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the gamma distribution and POISSON.DIST (GAMMA.DIST, GAMMADIST, GAMMA.INV, POISSON.DIST)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
gen-m9 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of the four incomplete-gamma distribution functions below, and every Excel cell the engine fails is
owned by a task.

## Context
- Gap triage 2026-11-04, second opus pass (`make eval` on main 1fb8465f, excel-graph; report.json
  `argument_coverage[].pairwise` and `unseen_pairs`; decision 0031 evidence gaps). Every failing cell of the run is
  named by an open task (the per-cell `owned` rule of `scripts/exit_owners.py`, deferred tasks included), so pairwise
  argument coverage is the evidence gap left. T-4009 leaves the 0.30 to 0.40 pairwise distributions to a later
  triage. This is their incomplete-gamma part: cdf, density and quantile through
  `functions/statistical/incomplete_gamma/`. T-4032 (normal and lognormal) and T-4034 (exponential and Weibull) take
  the rest. The sets are disjoint from T-4009's and T-4007's (POISSON, the legacy name, is T-4007's), so the batches
  can land in either order; whichever lands second re-reads report.json before generating.
- Functions (pairwise share, unseen pairs at 1fb8465f; catalog `milestone` M9, `prior` 0.05): GAMMA.DIST 0.34 (482),
  GAMMADIST 0.32 (496), GAMMA.INV 0.33 (245), POISSON.DIST 0.36 (232). 1,455 unseen pairs in all. First unseen pairs:
  GAMMA.DIST x/alpha = (number, empty_text), (zero, negative); GAMMA.INV probability/alpha = (number, zero),
  (number, text); POISSON.DIST x/mean = (number, empty_text), (zero, numeric_text).
- Dry run on 1fb8465f into a scratch suite that was then deleted: `cd tools && uv run rustsheet-gen --policy
  arg-pairs --milestone M9 --functions GAMMA.DIST,GAMMADIST,GAMMA.INV,POISSON.DIST --suite <scratch>` wrote 16 cases
  with 2,910 formulas.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates (search
  `rg '<FUNCTION>' docs/tasks/M9` first):
  - single argument kinds: T-3127, T-3128 (GAMMADIST) and T-3185;
  - incomplete-gamma kernels: T-2367, T-3219, T-3224, T-3234, T-3242, T-3469 and T-4000 (cdf and density); T-3265,
    T-3294, T-3146 (blocked) and T-3232 (quantiles); T-3247, T-3295 and T-3366 (POISSON.DIST); T-3252 (research probe
    of the large-shape prefactor window); Gate A tables T-3925 (GAMMA.DIST) and T-3863 (GAMMA.INV huge shapes).
  Numeric residue of these kernels goes to those tasks. A new last-digit shape gets a `defer: phase-1b` task, and a
  numeric-fitting one says "Needs an oracle sweep" and lists the R4 guards below.
- Why opus: the cells mix argument-check rules with points deep in the incomplete-gamma kernel, where this lane has
  many fit tasks and several time-boxed ones. The rules include a zero alpha or beta against a text or empty-text x,
  POISSON.DIST's truncation of a fractional or numeric-text x, and the legacy GAMMADIST's check order against
  GAMMA.DIST's. Sorting each failing cell to the right owner takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- Own each failing cell in the same PR: an existing open task that covers the rule, or a new task via
  `make new-task` in this lane. A wrong type, error or text gets `campaign: semantics` and a
  `## Visible-parity stage` section; name the case file stem and the A1 cells in one bullet (decision 0159). Record
  any new Excel quirk in docs/excel-semantics/statistics.md, citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions` with the four functions above are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the four functions has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why.
- [ ] Failing cells are known failures in eval/baseline/gen-m9.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M9` and `make unowned-off ALL=1` report no new unowned cell
      from the batch.
- [ ] Any fit task filed for the batch's cells lists the R4 guards of reviewer-excel-fidelity check 7 (decision 0188)
      in its acceptance criteria: hold-out counts, fresh sweep, Gate A envelope result, `FITTED` tag and range,
      reproducibility floor, no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning tasks make them), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- GAMMAINV (0.27 pairwise, T-4009), POISSON (T-4007), the CHISQ functions (T-4009), the batches of T-4032 and T-4034,
  single argument-kind probes, and other milestones.
