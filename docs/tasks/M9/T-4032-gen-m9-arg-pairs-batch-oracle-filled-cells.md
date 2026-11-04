---
id: T-4032
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the normal and lognormal distributions (NORM.DIST, NORMDIST, NORM.INV, NORMINV, NORM.S.DIST, LOGNORM.DIST, LOGNORMDIST, LOGNORM.INV)"
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
argument coverage of the eight normal and lognormal distribution functions below, and every Excel cell the engine
fails is owned by a task.

## Context
- Gap triage 2026-11-04, second opus pass (`make eval` on main 1fb8465f, excel-graph; report.json
  `argument_coverage[].pairwise` and `unseen_pairs`; decision 0031 evidence gaps). Every failing cell of the run in
  gen-m9, ironcalc-calc, probes and probes-numeric is named by an open task (the per-cell `owned` rule of
  `scripts/exit_owners.py`, deferred tasks included), `make phase1-exit` reports 0 milestone exit cells without an
  owner of their milestone, and the evidence floor is met. Pairwise argument coverage is the evidence gap left.
- T-4009 (the 29 continuous distributions below 0.3 pairwise) leaves "the distributions at 0.30 to 0.40 pairwise
  (NORM*, LOGNORM*, GAMMA.DIST, GAMMA.INV, GAMMADIST, EXPON.DIST, EXPONDIST, WEIBULL, POISSON.DIST)" to a later
  triage. This task is the normal and lognormal part. T-4033 (gamma and POISSON.DIST) and T-4034 (exponential and
  Weibull) take the rest. The function sets are disjoint from T-4009's, so the batches can land in either order;
  whichever lands second re-reads report.json before generating.
- Functions (pairwise share, unseen pairs at 1fb8465f; catalog `milestone` M9, `prior` 0.1 for the NORM names and
  0.05 for LOGNORM): NORM.DIST 0.33 (486), NORMDIST 0.32 (493), NORM.INV 0.31 (251), NORMINV 0.31 (251), NORM.S.DIST
  0.32 (82), LOGNORM.DIST 0.37 (461), LOGNORMDIST 0.31 (249), LOGNORM.INV 0.36 (233). 2,506 unseen pairs in all.
  First unseen pairs: NORM.DIST x/mean = (number, empty_text), (number, boolean); NORMINV probability/mean =
  (number, numeric_text), (number, text); NORM.S.DIST z/cumulative = (number, numeric_text).
- Dry run on 1fb8465f into a scratch suite that was then deleted: `cd tools && uv run rustsheet-gen --policy
  arg-pairs --milestone M9 --functions NORM.DIST,NORMDIST,NORM.INV,NORMINV,NORM.S.DIST,LOGNORM.DIST,LOGNORMDIST,LOGNORM.INV
  --suite <scratch>` wrote 28 cases with 5,012 formulas.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates (search
  `rg '<FUNCTION>' docs/tasks/M9` first):
  - single argument kinds of these parameters: T-3127 (current names) and T-3128 (NORMDIST, NORMINV, LOGNORMDIST),
    both research and sonnet, and T-3185. This batch covers the pairs;
  - kernels: T-0925 (inverse normal AS241: NORM.S.INV, NORM.INV, LOGNORM.INV), T-0975 (normal pdf last digits),
    T-0968 (blocked: normal cdf and erfc residue); Gate A tables T-3783 (NORM*) and T-3785 (LOGNORM*).
  Numeric residue of these kernels goes to those tasks. A new last-digit shape gets a `defer: phase-1b` task, and a
  numeric-fitting one says "Needs an oracle sweep" and lists the R4 guards below.
- Why opus: the batch's cells mix argument-check rules with points in the erfc and inverse-normal kernels that
  already have fit tasks. The rules include which of two bad arguments wins, whether a zero or negative standard_dev
  is `#NUM!` before a text mean is `#VALUE!`, whether the legacy names check or coerce in another order than the
  dotted ones, and how a numeric-text or boolean `cumulative` reads. Sorting each failing cell to the right owner, and
  not filing fits blind, takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/statistical/normal.rs` and its
  callers). A wrong type, error or text gets `campaign: semantics` and a `## Visible-parity stage` section; name the
  case file stem and the A1 cells in one bullet (decision 0159). Record any new Excel quirk in
  docs/excel-semantics/statistics.md, citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions` with the eight functions above are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the eight functions has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to
      load).
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
- The gamma and POISSON.DIST batch (T-4033), the exponential and Weibull batch (T-4034), the continuous
  distributions below 0.3 pairwise (T-4009), the discrete distributions (T-4007), single argument-kind probes (T-3127,
  T-3128, T-3185), NORM.S.INV, NORMSDIST and NORMSINV (one argument each, so no pairs), and other milestones.
