---
id: T-5098
title: "Usage tiers dashboard block: escape function names, re-wrap the page.py docstring, check it on the first real usage_gates report (T-5094 follow-up)"
milestone: M0
lane: quality
status: todo
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
- [ ] `failing_table` (and any other cell built from report text in usage.py) escapes with `md_text`, with a test using a name that needs escaping.
- [ ] The page.py docstring is wrapped like its neighbours.
- [ ] After the first full `make eval` with `usage_gates`, `make progress-report` renders the block without "not measured", and its tier rows match report.json `usage_gates` and `make phase1-exit`'s tier lines; the Outcome records both lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the gate computation (gate_tiers.rs, usage_gates.rs).
