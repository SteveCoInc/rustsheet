---
id: T-4034
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the exponential and Weibull distributions (EXPON.DIST, EXPONDIST, WEIBULL)"
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
argument coverage of EXPON.DIST, EXPONDIST and WEIBULL, and every Excel cell the engine fails is owned by a task.

## Context
- Gap triage 2026-11-04, second opus pass (`make eval` on main 1fb8465f, excel-graph; report.json
  `argument_coverage[].pairwise` and `unseen_pairs`; decision 0031 evidence gaps). Every failing cell of the run is
  named by an open task, so pairwise argument coverage is the evidence gap left. T-4009 leaves the 0.30 to 0.40
  pairwise distributions to a later triage. This is their closed-form exponential and Weibull part. T-4032 (normal
  and lognormal) and T-4033 (gamma and POISSON.DIST) take the rest. WEIBULL.DIST (0.29 pairwise) is T-4009's; the
  legacy WEIBULL is here. The sets are disjoint, so the batches can land in either order; whichever lands second
  re-reads report.json before generating.
- Functions (pairwise share, unseen pairs at 1fb8465f; catalog `milestone` M9, `prior` 0.05): EXPON.DIST 0.39 (220),
  EXPONDIST 0.32 (246), WEIBULL 0.34 (478). 944 unseen pairs in all. Single kinds are still missing too: EXPONDIST x
  and lambda have 9 and 8 of 11, and WEIBULL x, alpha and beta have 9 of 11 each. The legacy names have no
  numeric-text or empty-text cells yet. First unseen pairs: EXPON.DIST x/lambda = (number, empty_text),
  (number, error); EXPONDIST x/lambda = (number, numeric_text); WEIBULL x/alpha = (number, empty_text),
  (number, error).
- Dry run on 1fb8465f into a scratch suite that was then deleted: `cd tools && uv run rustsheet-gen --policy
  arg-pairs --milestone M9 --functions EXPON.DIST,EXPONDIST,WEIBULL --suite <scratch>` wrote 11 cases with 1,888
  formulas.
- Related open tasks: T-3127 and T-3128 (single kinds, EXPONDIST and WEIBULL among them; research, sonnet) and
  T-3185; T-0978 (WEIBULL.DIST last digits); Gate A table T-3785 (EXPON.DIST, WEIBULL.DIST). Give new cells of their
  shapes to them.
- Why opus: the closed forms leave little numeric residue, so most failing cells will be argument-check rules. Two
  examples: whether a zero lambda or alpha is `#NUM!` before a text or empty-text x is `#VALUE!`, and whether the
  legacy names coerce numeric text and booleans as the dotted ones do. Each needs a ruling on which existing task, if
  any, owns the rule and which semantics entry it extends. A numeric miss at an extreme x needs its Gate A envelope
  reading before it is filed as a fit or as Phase 1b.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- Own each failing cell in the same PR: an existing open task, or a new task via `make new-task` in this lane
  (`functions/statistical/`). A wrong type, error or text gets `campaign: semantics` and a `## Visible-parity stage`
  section; name the case file stem and the A1 cells in one bullet (decision 0159). Record any new Excel quirk in
  docs/excel-semantics/statistics.md, citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions EXPON.DIST,EXPONDIST,WEIBULL` are committed with their
      excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the three functions has a `pairwise` share of at least 0.9 in report.json
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
- WEIBULL.DIST (T-4009), the batches of T-4032 and T-4033, single argument-kind probes, and other milestones.
