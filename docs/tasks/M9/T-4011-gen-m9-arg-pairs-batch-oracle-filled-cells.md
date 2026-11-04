---
id: T-4011
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 odd-period bond functions (ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD)"
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
argument coverage of the four M9 odd-period bond functions, and every Excel cell the engine fails is owned by a task.

## Context
- Gap triage 2026-11-04 (report of main cd41e050; report.json `argument_coverage[].pairwise` and `unseen_pairs`;
  decision 0031 evidence gaps). Every failing Excel cell of the eval has an open owner (`make unowned-off ALL=1`
  lists only cells T-3978 owns and cells owned by deferred tasks), and all four are above the evidence floor, but
  four fifths of their argument-class pairs have no Excel cell. ODDFYIELD and ODDFPRICE have the most unseen pairs of
  any function, ODDLYIELD and ODDLPRICE the fourth and fifth most (eight or nine parameters each), and no batch task covers them (T-3779, T-3780, T-3782 and T-4010
  cover the other financial families).
- Pairwise share (unseen pairs) at cd41e050: ODDFPRICE 0.19 (3542); ODDFYIELD 0.18 (3580); ODDLPRICE 0.19 (2760);
  ODDLYIELD 0.18 (2785). 12,667 unseen pairs in all; catalog `prior` 0.05 each. First unseen pairs: ODDFPRICE
  settlement/maturity = (number, empty_text), (number, error); ODDFYIELD settlement/maturity = (number,
  numeric_text), (number, empty_text).
- Dry run on cd41e050 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions ODDFPRICE,ODDFYIELD,ODDLPRICE,ODDLYIELD`
  wrote 128 cases (about 25,300 calls, no pair skipped). Split it into two PRs if oracle fill or eval time needs it
  (ODDFPRICE/ODDFYIELD, then ODDLPRICE/ODDLYIELD), each meeting the criteria below.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3184 (research, opus) and T-3125 (research, sonnet) cover the single argument kinds of these parameters; this
    batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known owners: T-3031 (ODDFYIELD's slope for a long odd first period), T-3043 (ODDFYIELD last ulps, deferred),
    T-3625 (ODDFYIELD #NUM! on the 100th Newton check); T-2483 (done) recorded the argument check order and
    ODDLPRICE before the last coupon (financial.md). A last-digit cell of a shape a deferred fit task already owns
    goes to that task; a new last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs
    an oracle sweep" and lists the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: these functions check four date orderings (settlement, maturity, issue or last interest, first coupon),
  truncate serials, validate frequency and basis, and the yield functions iterate; a new failing cell may be a
  check-order rule T-2483 did not reach, a coercion rule, or a solver or day-count residue, and sorting them into
  owners (and not filing fits blind) takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/financial/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (financial.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions ODDFPRICE,ODDFYIELD,ODDLPRICE,ODDLYIELD` are committed
      with their excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the four functions has a `pairwise` share of at least 0.9 in report.json
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
- The annuity (T-3779), cash-flow and depreciation (T-3780), coupon and bond (T-3782) and discount, maturity and
  T-bill (T-4010) batches, single argument-kind probes (T-3184, T-3125), and other milestones.
