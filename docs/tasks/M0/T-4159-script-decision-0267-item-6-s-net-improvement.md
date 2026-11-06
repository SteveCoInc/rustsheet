---
id: T-4159
title: "Script decision 0267 item 6's net-improvement check for family regressions (counts before/after from two report.json files)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A script turns decision 0267 item 6's net-improvement test into numbers. It takes two report.json files, main's and
the branch's, and checks two conditions. First, every regressed cell is a number of a `THIRTEEN_DIGIT_FAMILY` formula
that stays in `residue13` or better, and no other function or verdict type regresses. Second, the family's cells
within 15 digits rise and its visibly-off numbers do not. It exits non-zero when either check fails, and prints the
counts a PR body cites.

## Context
- Decision 0267 item 6 lets a PR record family regressions with `update-baseline --allow-regressions` under
  conditions (a) to (d). Today only the Excel-fidelity reviewer checks them, from the PR body's arithmetic. The #3137
  review (round 3, MINOR) asked for a tool, so the check can be reproduced.
- Inputs: `rustsheet-eval run` writes report.json, with `cases[].cells[]` (expected, actual, verdict, formula) and
  `functions[].closeness`. Family membership follows `closeness::THIRTEEN_DIGIT_FAMILY` and `blamed`. The script must
  not re-implement value parsing (CLAUDE.md conventions): read the report's values.
- The same review asked that the statistics.md entry "Incomplete-beta family: Excel's 14th and 15th digits are noise"
  repeat in its Status line that the 1% figure covers only cells already visibly off. Fold that one-line docs fix into
  this PR.

## Acceptance criteria
- [x] `scripts/` or `tools/` holds the check, with tests: a clean improvement, a non-family regression, a regression
      past the 13th digit, and no net gain.
- [x] Decision 0267 item 6 (d) names the script, and the statistics.md Status line carries the caveat.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing item 6's conditions.

## Outcome
- `scripts/family_net.py MAIN_REPORT BRANCH_REPORT` checks decision 0267 item 6 (a) to (c) and prints the counts (d)
  cites: the family's within-15-digit, `visible_numeric` and `residue13` totals from `functions[].closeness`, main to
  branch, and each regressed cell with its bucket before and after. Exit 0 when all hold, 1 when one fails, 2 when the
  reports cannot be compared. Decision 0309 records the new script (protected path); decision 0267 item 6 (d) names it.
- One definition kept: report.json now writes each listed cell's `closeness` bucket and `thirteen_digit_family` flag
  (`closeness::serialize_cells`, from the same `bucket` and `in_thirteen_digit_family`), and the family list at the top
  level. The script derives nothing itself. An unlisted cell counts as bit-exact, and each case's `checked - passed -
  deviations` must equal its listed scored cells (checked on a real gen-m9 run: all 344 cases hold).
- Tests: `scripts/test_family_net.py` (11 tests, which cover a clean improvement, a non-family regression, a regression past
  the 13th digit, a number turning into an error, no net gain, a text regression, a case that stops running, new
  coverage left uncompared, unsupported cells, reports that cannot be compared, and the exit codes); Rust
  `closeness::tests::listed_cells_carry_their_bucket_and_family_flag`.
- statistics.md "Incomplete-beta family: Excel's 14th and 15th digits are noise": its Status line now says the 1% figure
  covers only cells already visibly off.
- Eval delta: none (report.json format only; every baseline unchanged).
