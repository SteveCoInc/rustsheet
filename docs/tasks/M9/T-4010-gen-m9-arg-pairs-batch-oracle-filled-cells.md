---
id: T-4010
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 discount, maturity and T-bill securities functions (ACCRINT, ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT, TBILLEQ, TBILLPRICE, TBILLYIELD)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
gen-m9 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of the 12 M9 discount, maturity and T-bill securities functions below, and every Excel cell the
engine fails is owned by a task.

## Context
- Gap triage 2026-11-04 (report of main cd41e050; report.json `argument_coverage[].pairwise` and `unseen_pairs`;
  decision 0031 evidence gaps). Every failing Excel cell of the eval has an open owner (`make unowned-off ALL=1`
  lists only cells T-3978 owns and cells owned by deferred tasks), and every function below is above the evidence
  floor, but more than three quarters of their argument-class pairs have no Excel cell. T-3779 (annuities), T-3780
  (cash flows and depreciation) and T-3782 (coupon and bond functions) cover the other M9 financial batches; T-3779
  names "the other securities functions" as out of scope, and no task covers them.
- Pairwise share (unseen pairs) at cd41e050: ACCRINT 0.15 (2878); TBILLYIELD 0.18 (298); DISC 0.23 (935); INTRATE
  0.23 (934); PRICEDISC 0.23 (929); PRICEMAT 0.23 (1390); YIELDDISC 0.23 (930); TBILLEQ 0.23 (278); TBILLPRICE 0.23
  (279); ACCRINTM 0.24 (925); YIELDMAT 0.24 (1386); RECEIVED 0.25 (907). 12,069 unseen pairs in all; catalog `prior`
  0.05 each. First unseen pairs: ACCRINT issue/first_interest = (number, numeric_text), (number, empty_text);
  ACCRINTM issue/settlement = (number, empty_text), (number, error).
- Dry run on cd41e050 into a scratch suite that was then deleted: `cd tools && uv run rustsheet-gen --policy
  arg-pairs --milestone M9 --functions` with the 12 names above wrote 126 cases (about 24,100 calls, no pair
  skipped; ACCRINT alone is about a quarter). Split it into two PRs if oracle fill or eval time needs it (ACCRINT,
  ACCRINTM, PRICEMAT, YIELDMAT; then the discount and T-bill functions), each meeting the criteria below.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3184 (research, opus) and T-3125 (research, sonnet) cover the single argument kinds of these parameters; this
    batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known owners: T-1374 (bond and T-bill values a few ulps off, deferred), T-3782 (shared settlement/maturity/basis
    readers of the coupon functions); T-2487 (done) recorded the T-bill error order and the settlement below serial
    1 rule (financial.md). A last-digit cell of a shape a deferred fit task already owns goes to that task; a new
    last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle sweep" and lists
    the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: these functions truncate date serials, validate basis and frequency, and order their checks
  (settlement against maturity or issue, the T-bill one-year limit, a non-positive rate or price) differently from
  one another; ACCRINT also reads calc_method as a boolean. A new failing cell may be a date-truncation or coercion
  rule, an error-order rule or a day-count residue, and sorting them into owners takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/financial/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (financial.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions` with the 12 functions above are committed with their
      excel-graph oracle sidecars and 0 fill failures.
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
- The annuity (T-3779), cash-flow and depreciation (T-3780), coupon and bond (T-3782) and odd-period bond (T-4011)
  batches, single argument-kind probes (T-3184, T-3125), and other milestones.
