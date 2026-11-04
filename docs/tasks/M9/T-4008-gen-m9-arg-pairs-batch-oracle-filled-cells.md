---
id: T-4008
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 hypothesis tests and least-squares functions (F.TEST, FTEST, T.TEST, TTEST, Z.TEST, ZTEST, CHISQ.TEST, CHITEST, TREND, GROWTH, LINEST, LOGEST)"
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
argument coverage of the 12 M9 hypothesis-test and least-squares functions below, and every Excel cell the engine
fails is owned by a task.

## Context
- Gap triage 2026-11-04 (report of main cd41e050; report.json `argument_coverage[].pairwise` and `unseen_pairs`;
  decision 0031 evidence gaps). Every failing Excel cell of the eval has an open owner (`make unowned-off ALL=1`
  lists only cells T-3978 owns and cells owned by deferred tasks), and every function below is above the evidence
  floor, but four fifths of their argument-class pairs have no Excel cell. T-3781 (done) left these functions out
  of its batch ("LINEST/LOGEST/TREND/GROWTH" in its out of scope). TREND, GROWTH, LINEST and LOGEST have catalog
  `prior` 0.1, the highest of the remaining M9 statistical gap with the binomial functions (T-4007).
- Pairwise share (unseen pairs) at cd41e050: TREND 0.14 (627); F.TEST 0.17 (100); GROWTH 0.17 (604); TTEST 0.18
  (594); Z.TEST 0.18 (298); LINEST 0.19 (587); LOGEST 0.19 (586); T.TEST 0.21 (577); ZTEST 0.21 (286); FTEST 0.23
  (93); CHISQ.TEST 0.23 (93); CHITEST 0.26 (89). 4,534 unseen pairs in all. First unseen pairs: F.TEST and FTEST
  array1/array2 = (number, zero), (number, numeric_text).
- Dry run on cd41e050 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions F.TEST,FTEST,T.TEST,TTEST,Z.TEST,ZTEST,CHISQ.TEST,CHITEST,TREND,GROWTH,LINEST,LOGEST`
  wrote 48 cases (about 9,100 calls, no pair skipped; three quarters of them TREND, GROWTH, LINEST, LOGEST, T.TEST
  and TTEST). LINEST, LOGEST, TREND and GROWTH spill: generate with `--spill`.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3185 (research, opus), T-3129 and T-3369 (research, sonnet) cover the single argument kinds and the data-array
    kinds of these functions; this batch covers the pairs. Whichever lands second re-reads report.json first.
  - Known owners: T-2006 (least-squares evaluation order, deferred), T-2690 (LINEST/LOGEST/GROWTH visible parity),
    T-2574 (LINEST lift and error order), T-2559 (TREND and TTEST over boolean results), T-2930, T-3254 and T-3730
    (T.TEST/TTEST), T-2655 (TTEST tails 1E15), T-2765 (F.TEST last digits, deferred), T-3044 and T-2650 (CHISQ.TEST),
    T-1689 (CHISQ.TEST blanks probe), T-2775 (Z.TEST). A last-digit cell of a shape a deferred fit task already owns
    goes to that task; a new last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs
    an oracle sweep" and lists the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: these functions take whole arrays, drop or refuse non-numbers by their own rules (T.TEST pairs, the
  least-squares X/Y shape checks, const and stats flags), and spill; a new failing cell may be a shape or
  error-order rule, a spill-extent difference, or a residue of the least-squares or Student-t kernels, and sorting
  them into owners takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate with `--spill`; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/statistical/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (statistics.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --spill --functions` with the 12 functions above are committed with
      their excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 12 functions has a `pairwise` share of at least 0.9 in report.json
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
- The distributions (T-4007, T-4009), the FORECAST.ETS family (M14 owner T-2425), single argument-kind probes
  (T-3185, T-3129, T-3369), and other milestones.
