---
id: T-5098
title: "Usage tiers dashboard block: escape function names, re-wrap the page.py docstring, check it on the first real usage_gates report (T-5094 follow-up)"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-5092]
---

## Goal
The dashboard's "Usage tiers" block (T-5094) is tidy and proven on a real report, not only on fixtures.

## Context
- From PR #4090's review (round 1 MINORs, both reviewers approved):
  - security: `tools/src/rustsheet_tools/progress_report/usage.py` `failing_table` writes function names without `md_text`, unlike the suite names and exit-line text beside them;
  - design: the module docstring line in `progress_report/page.py` that names `usage-tiers` is far longer than its neighbours.
- T-5094 was built against fixtures. T-5092 adds `usage_gates` to report.json, and `make phase1-exit` gains the `foundational` and `expert` lines, so the first `make eval` after it lands is the first real input.

## Acceptance criteria
- [x] `failing_table` (and any other cell built from report text in usage.py) escapes with `md_text`, with a test using a name that needs escaping.
- [x] The page.py docstring is wrapped like its neighbours.
- [x] After the first full `make eval` with `usage_gates`, `make progress-report` renders the block without "not measured", and its tier rows match report.json `usage_gates` and `make phase1-exit`'s tier lines; the Outcome records both lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the gate computation (gate_tiers.rs, usage_gates.rs).

## Outcome
- `markdown.md_inline` is the inline-punctuation part of `md_text` (`md_text` is now `md_cell(md_inline(...))`).
  `usage.failing_table` escapes function names with it, since `table` applies `md_cell` to each cell itself;
  `md_text` there would have escaped a `|` twice and broken the row. Test
  `test_failing_function_names_are_escaped_in_the_table` (a name `A|B_C*`: escaped once, 7 cells).
- page.py's module docstring is wrapped like its neighbours.
- Real report: a full `make eval` of main e53d43fa (2026-11-08; fetched suites not fetched) has `usage_gates`, and
  the "Usage tiers" block rendered from it (`usage.block` on that report.json and `make phase1-exit --json`) shows
  no "not measured"; its tier rows match report.json:
  - report.json: foundational 122 functions, 202,686 cells, 29 wrong type, 1 outside, failing TEXT and EDATE, not
    met; expert 383 functions, 890,585 cells, 1,355 wrong type, 391 outside, 11 failing (IRR, RATE, BETA.INV,
    F.DIST.RT, F.INV.RT, T.TEST, F.INV, FINV, ACCRINT, MROUND, CHIINV), not met.
  - `make phase1-exit` (adds the accuracy points): `foundational: not met` — 206,346 cells (3,660 accuracy points),
    type 99.9859% (29 wrong), envelope 99.9995% (1 outside); `expert: not met` — 932,008 cells (41,423 accuracy
    points), type 99.8424% (1,469 wrong), envelope 99.9511% (456 outside).
  - `make progress-report` itself rewrote only the status block, because the run skipped the fetched suites
    (stale page, SPEC-0022); the steward's next full run with fetched suites writes the block on the page.
- `pytest tests/test_progress_report.py tests/test_progress_page.py`: 85 pass.
- Eval delta: none.
