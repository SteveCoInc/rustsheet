---
id: T-3809
title: "gen-m5 arg-pairs batch for TEXT: oracle-filled cells for the 52 unseen value x format_text class pairs, with format-code triage of each failing cell"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
gen-m5 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` that covers TEXT's unseen
(value, format_text) argument-class pairs. Each failing cell it finds is triaged to the rule that causes it
(TEXT's argument coercion here, or a number-format rule in an M10 task) and owned.

## Context
- Gap triage 2026-11-02 (`make eval` on main 2290b8769, report.json `argument_coverage`; decision 0031 evidence
  gaps). TEXT (M5, prior 2) has 11849 Excel cells, 11784 passing. The 65 failing ones all have open owners (M10
  number-format tasks; `make unowned-off ALL=1`). Its pairwise share is 0.57: 52 of the 121 (value, format_text)
  class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank, range;
  crates/rustsheet-eval/src/argument_coverage.rs) have no Excel cell. format_text has never been an error (10 of
  11 classes). The unseen pairs are mostly error, blank and range values against every format class, and number,
  zero, empty_text and range format_text (a format code given as a number, `TEXT(x,0)`).
- T-3512 (the M5 number-to-text batch) left TEXT out on purpose: "its format-code argument belongs to the M10 number-format
  tasks, and a pair batch there needs format-code triage". That triage is why this task is opus. A failing cell may
  come from TEXT's own coercion of a non-text format_text (number to code text, boolean, blank, error order between
  the two arguments, range lifting), which is this lane's code in crates/rustsheet-engine/src/functions/text/
  (`fn_text`). Or it may come from how a format code renders a value (number-formats.md, the formatter under
  crates/rustsheet-engine/src/formatter/), which belongs to the open M10 TEXT tasks (T-2230, T-2798, T-3089,
  T-3342, T-3598, T-3710, T-3711, T-3718) or to T-3649 (the M5 sweep of which code/value pairs are `#VALUE!`).
  Hand each cell to the matching owner, or file a new task in this lane for a coercion rule. A new task gets
  `campaign: semantics` and a `## Visible-parity stage` section for a wrong type, error or text, and its case file
  stem and A1 names go in one bullet (decision 0159).
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M5 --functions TEXT`
  (tools/src/rustsheet_tools/gen/arg_pairs.py; T-3493 checked that `--functions` is honoured). Fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5` and score. TEXT is scalar on scalar arguments, and a
  forced `range` lifts to at most 2 x 2, which the policy's row stride leaves room for.
- M5's exit measures gen-m5 (docs/milestones.md, decision 0085), and the Phase 1 rate counts gen-* cells (decision
  0188), so every failing cell needs an owner in the same PR. Record any new Excel quirk in
  docs/excel-semantics/text.md or number-formats.md, citing the cells.

## Acceptance criteria
- [x] New gen-m5 cases from `--policy arg-pairs --milestone M5 --functions TEXT` with their excel-graph oracle
      sidecars are committed.
- [x] After `make eval`, TEXT's `pairwise` share in report.json `argument_coverage` is at least 0.9, and format_text
      has an error cell, or the PR lists the pairs left unseen and why.
- [x] Failing cells are known failures in eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions), each listed
      by an open task (an existing M10 or T-3649 owner or a new one filed here), and `make exit-owners MILESTONE=M5`
      and `make unowned-off ALL=1` report no unowned cell from the batch.
- [x] The PR description gives the triage: for each failing cell, coercion rule or format-code rule, and its owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the format-code rendering the batch finds (the M10 tasks and T-3649), beyond a one-line coercion fix in
  `fn_text` that the PR also tests.
- Other text functions (T-3510, T-3511, T-3512) and other milestones.

## Outcome
- Added gen-m5/0428ca72d3724bea (`rustsheet-gen --policy arg-pairs --milestone M5 --functions TEXT`, seed 0): 102 calls
  over the 51 (value, format_text) class pairs still unseen on main (one of the 52 listed at triage had been covered
  since), with its excel-graph sidecar (2026-11-02).
- Eval delta: gen-m5 +102 scored cells, all 102 pass (`make eval-show`: 102/102, 0 deviations); 0 regressions, 0 new
  known failures, so no triage owner and no new task was needed. TEXT's report.json `argument_coverage`: single
  0.95 → 1.0, pairwise 0.58 → 1.0, no unseen pair left; format_text now has error cells.
- Knowledge: docs/excel-semantics/text.md "TEXT's format_text of every argument class (T-3809)" records how a
  non-text code is read (number as its General text, blank as empty, boolean as the rejected code `TRUE`/`FALSE`) and
  the error order between the two arguments, citing the batch's cells.
