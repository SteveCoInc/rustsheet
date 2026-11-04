---
id: T-4009
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the 29 M9 continuous distributions and quantiles below 0.3 pairwise (BETA, F, T, CHISQ families, GAMMAINV, LOGINV, WEIBULL.DIST, CONFIDENCE*)"
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
argument coverage of the 29 M9 continuous distribution and quantile functions below, and every Excel cell the
engine fails is owned by a task.

## Context
- Gap triage 2026-11-04 (report of main cd41e050; report.json `argument_coverage[].pairwise` and `unseen_pairs`;
  decision 0031 evidence gaps). Every failing Excel cell of the eval has an open owner (`make unowned-off ALL=1`
  lists only cells T-3978 owns and cells owned by deferred tasks), and every function below is above the evidence
  floor (most have thousands of probe cells), but those cells are numeric sweeps at valid arguments: about three
  quarters of the argument-class pairs (an error, empty text, numeric text, a boolean or a blank in one argument
  beside a zero or negative in another) have no Excel cell.
- Functions (pairwise share, unseen pairs at cd41e050): BETA.DIST 0.24 (1375); BETA.INV 0.24 (919); BETADIST 0.25
  (904); BETAINV 0.20 (968); F.DIST 0.23 (561); F.DIST.RT 0.21 (285); FDIST 0.17 (303); F.INV 0.16 (304); F.INV.RT 0.17
  (300); FINV 0.16 (305); T.DIST 0.25 (274); T.DIST.RT 0.19 (98); T.DIST.2T 0.26 (89); TDIST 0.20 (291); T.INV 0.24
  (92); T.INV.2T 0.23 (93); TINV 0.21 (96); CHISQ.DIST 0.27 (266); CHISQ.DIST.RT 0.17 (100); CHIDIST 0.20 (97);
  CHISQ.INV 0.17 (101); CHISQ.INV.RT 0.20 (97); CHIINV 0.19 (98); GAMMAINV 0.27 (265); LOGINV 0.27 (264); WEIBULL.DIST
  0.29 (519); CONFIDENCE 0.21 (287); CONFIDENCE.NORM 0.26 (270); CONFIDENCE.T 0.26 (270). 9,891 unseen pairs in all.
  Catalog `prior` 0.05 each. First unseen pairs: BETA.DIST x/alpha = (number, zero), (number, numeric_text);
  BETA.INV probability/alpha = (number, zero), (number, negative).
- Dry run on cd41e050 into a scratch suite that was then deleted: `cd tools && uv run rustsheet-gen --policy
  arg-pairs --milestone M9 --functions` with the 29 names above wrote 109 cases (about 19,800 calls, no pair
  skipped). Split it into two PRs if oracle fill or eval time needs it (the BETA and F families, then the T, CHISQ
  and remaining functions), each meeting the criteria below.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3185 (research, opus), T-3127 and T-3128 (research, sonnet) cover the single argument kinds of these
    parameters; this batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known owners (many; search `rg '<FUNCTION>' docs/tasks/M9` before filing): T-3363 and T-0967/T-3061 (Student-t,
    F and chi-squared quantiles), T-3364 (extrapolated fits fall back to math), T-1871, T-3351, T-3388 and T-3862
    (BETA.INV and F.INV), T-3577 (BETA.DIST cdf 0 at tiny shapes), T-3554 and T-2585 (T.DIST density), T-3310 and
    T-3337 (CHISQ.DIST.RT and CHIINV tails), T-3079 (the T-0969 sweep cells). Most numeric residue of these kernels
    is owned by a deferred last-digit task: a cell of a shape one already owns goes to it; a new last-digit shape
    gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle sweep" and lists the R4 guards of
    reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: the batch's cells mix argument-check rules (which of two bad arguments wins, whether a zero or negative
  shape is `#NUM!` before a text argument is `#VALUE!`, legacy names that check in another order) with points
  that land in the incomplete-beta, incomplete-gamma and quantile kernels this lane has many fit tasks for. Sorting
  each failing cell to the right owner, and not filing fits blind, takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/statistical/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (statistics.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions` with the 29 functions above are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 29 functions has a `pairwise` share of at least 0.9 in report.json
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
- The discrete distributions (T-4007), the tests and least-squares functions (T-4008), the distributions at 0.30
  to 0.40 pairwise (NORM*, LOGNORM*, GAMMA.DIST, GAMMA.INV, GAMMADIST, EXPON.DIST, EXPONDIST, WEIBULL, POISSON.DIST at
  cd41e050; a later triage files their batch once these land), the FORECAST.ETS family (M14 owner T-2425), single
  argument-kind probes (T-3185, T-3127, T-3128), and other milestones.
