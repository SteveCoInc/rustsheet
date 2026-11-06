---
id: T-4159
title: "Script decision 0267 item 6's net-improvement check for family regressions (counts before/after from two report.json files)"
milestone: M0
lane: harness
status: in-progress
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
- [ ] `scripts/` or `tools/` holds the check, with tests: a clean improvement, a non-family regression, a regression
      past the 13th digit, and no net gain.
- [ ] Decision 0267 item 6 (d) names the script, and the statistics.md Status line carries the caveat.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing item 6's conditions.
