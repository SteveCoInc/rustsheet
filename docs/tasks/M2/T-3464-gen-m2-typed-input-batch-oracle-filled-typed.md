---
id: T-3464
title: "gen-m2 typed-input batch: oracle-filled typed entries from --policy typed-input, every failing cell owned"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3462, T-3463]
---

## Goal
gen-m2 gains an oracle-filled batch of typed entries from `rustsheet-gen --policy typed-input` (T-3462 numbers,
T-3463 dates and clocks), and every Excel cell of it the engine fails is owned by an open task.

## Context
- The policy and its strata are documented in docs/architecture/synthetic-generator.md (T-3462, T-3463). This task
  runs it, fills it and scores it, as T-3279 does for `--policy arg-pairs`.
- Steps: `make eval`; `cd tools && uv run rustsheet-gen --policy typed-input --seed 0` (the suite is gen-m2);
  `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite gen-m2`; `make eval`;
  `make eval-show CASE=gen-m2/<case>` for each failing case.
- M2's exit needs gen-m2 with no visibly-off cell (docs/milestones.md, decision 0135), so every failing cell
  holds M2 open until it is fixed. Own each one in this PR: an open task that covers the rule (add the cells to
  it), or a new engine task via `make new-task` (milestone M2, lane engine), with the case file stem and the A1
  names in one bullet (decision 0159). `make exit-owners MILESTONE=M2` must report 0.
- An Excel answer that no semantics entry records (a shape the grammar read or refused that the docs do not
  cover) goes into docs/excel-semantics/ (coercion.md "Typed currency and percent formats", dates.md, numbers.md)
  in this PR, with the case and cells as evidence (CLAUDE.md directive 6).
- Entries Excel for the web refused (PATCH 400) are blank in Excel: they are answers, not fill failures. List
  how many there were in the PR.

## Acceptance criteria
- [x] The typed-input cases with their excel-graph oracle sidecars are committed under eval/suites/gen-m2/.
- [x] Failing cells are known failures in eval/baseline/gen-m2.txt (`make eval-baseline`, 0 regressions), and
      `make exit-owners MILESTONE=M2` reports 0.
- [x] The PR lists the visibly-off count per stratum and every task it filed or extended.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells the batch finds (the owning tasks).
- Changing the policy (follow-up to T-3462 or T-3463 if it draws something wrong).

## Outcome
`rustsheet-gen --policy typed-input --seed 0` wrote 2 gen-m2 cases, `7d9ef4824d1c5286` (seed 0, case 0) and
`361fd843151f1560` (case 1), with 240 entries each in A:D. `rustsheet-oracle fill -b excel-graph --suite gen-m2` filled
both sidecars: 0 failures, 0 skipped. Excel for the web refused 0 entries (no PATCH 400). The engine passes 1,424 of the
1,444 scored cells. Column D, `CELL("format")`, is `#VALUE!` on excel-graph and is not scored.

Visibly-off cells per stratum (20 in all):
- `sign/double-sign`, 12 cells: 4 entries (`+-122`, `+-0.7`, `-+584.17`, `++78.2218`) × A:C. Owner: T-5017 (new).
- `spacing/spaced`, 5 cells: B of `\t-3`, CHAR(160)`0`, `\t\t-701.80`, `-92598\t\t` and `-2.88\t`. Owner: T-5016 (new).
- `e-notation/star-star`, 2 cells: B and C of `39**9`. Owner: T-5017.
- `clock/near-miss`, 1 cell: C of `10:`. Owner: T-5017.
No other stratum (plain, currency, percent, fraction, the other e-notation and near-miss variants, dates, clocks) has
a visibly-off cell.

New quirks, recorded in coercion.md: "Number text padded with a TAB or CHAR(160) is not a number (T-3464)", and
"Typed entries: a doubled sign, a `**` exponent and `10:` are numbers (T-3464)". The failing cells are `F` lines in
eval/baseline/gen-m2.txt (`make eval-baseline`, 0 regressions), and `make exit-owners MILESTONE=M2` reports 0.
Tasks filed: T-5016 (P2, sonnet) and T-5017 (P2, sonnet). None extended.
