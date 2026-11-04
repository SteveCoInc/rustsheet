---
id: T-4007
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 discrete distributions (BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, BINOM.INV, CRITBINOM, NEGBINOM.DIST, NEGBINOMDIST, HYPGEOM.DIST, HYPGEOMDIST, POISSON)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
gen-m9 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of the 10 M9 discrete distribution functions below, and every Excel cell the engine fails is owned
by a task.

## Context
- Gap triage 2026-11-04 (report of main cd41e050; report.json `argument_coverage[].pairwise` and `unseen_pairs`;
  decision 0031 evidence gaps). Every failing Excel cell of the eval has an open owner (`make unowned-off ALL=1`
  lists only cells T-3978 owns and cells owned by deferred tasks), and every function below is above the evidence
  floor, but about three quarters of their argument-class pairs have no Excel cell. T-3781 (done) ran this batch for
  the order, rank and paired-regression statistics and found 154 failing cells in four new rules; no task covers the
  distributions. This group has the highest catalog `prior` of the M9 distribution gap (BINOM.DIST and BINOMDIST
  0.1).
- Pairwise share (unseen pairs) at cd41e050: BINOM.DIST 0.29 (514); BINOMDIST 0.29 (519); BINOM.DIST.RANGE 0.21
  (573); BINOM.INV 0.28 (263); CRITBINOM 0.23 (279); NEGBINOM.DIST 0.26 (538); NEGBINOMDIST 0.20 (291); HYPGEOM.DIST
  0.22 (949); HYPGEOMDIST 0.17 (603); POISSON 0.29 (257). 4,786 unseen pairs in all. First unseen pairs: BINOM.DIST
  number_s/trials = (number, empty_text), (zero, negative); BINOMDIST number_s/trials = (number, error).
- Dry run on cd41e050 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions BINOM.DIST,BINOMDIST,BINOM.DIST.RANGE,BINOM.INV,CRITBINOM,NEGBINOM.DIST,NEGBINOMDIST,HYPGEOM.DIST,HYPGEOMDIST,POISSON`
  wrote 53 cases (about 9,600 calls, no pair skipped).
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3185 (research, opus), T-3127 and T-3128 (research, sonnet) cover the single argument kinds of these
    parameters; this batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known owners: T-2146 (BINOM/NEGBINOM last digits), T-3366 (POISSON.DIST and BINOM.DIST on fresh points: a new
    numeric miss of a binomial or Poisson kernel is evidence for it), T-3959 and T-3961 (binomial cdf at a large n),
    T-3576 (NEGBINOM.DIST cdf #NUM!), T-2481, T-3235, T-3256 and T-1896 (HYPGEOM), T-2809 and T-2027 (BINOM.INV and
    CRITBINOM), T-3740 (BINOM.DIST.RANGE past the mean), T-3392 (Loader pmf, deferred), T-3469, T-3681 and T-3247
    (Poisson tails). A last-digit cell of a shape a deferred fit task already owns goes to that task; a new
    last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle sweep" and lists
    the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: the integer arguments truncate, the cumulative flag coerces, and each function checks its arguments in
  its own order (number_s against trials, sample against population), so a new failing cell may be a truncation
  or coercion rule, an error-order rule, or a numeric residue of one of the fitted kernels above. Telling them apart
  (and not filing fits blind) takes judgment beyond a mechanical run.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/statistical/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (statistics.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions` with the 10 functions above are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 10 functions has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to
      load).
- [ ] Failing cells are known failures in eval/baseline/gen-m9.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M9` and `make unowned-off ALL=1` report no new unowned cell
      from the batch.
- [ ] Any fit task filed for the batch's cells lists the R4 guards of reviewer-excel-fidelity check 7 (hold-out
      counts, fresh sweep, Gate A envelope result, `FITTED` tag and range, reproducibility floor, no keep-matched
      branch) in its acceptance criteria.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning tasks make them), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- The continuous distributions and quantiles (T-4009), the tests and least-squares functions (T-4008), POISSON.DIST
  (0.36 pairwise at cd41e050) and the other distributions at 0.30 to 0.40 that T-4009's out of scope lists, single
  argument-kind probes (T-3185, T-3127, T-3128), and other milestones.
