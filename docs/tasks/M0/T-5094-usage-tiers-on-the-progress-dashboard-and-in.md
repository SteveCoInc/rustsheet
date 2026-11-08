---
id: T-5094
title: "Usage tiers on the progress dashboard and in the steward's daily status (SPEC-0037 step 4)"
milestone: M0
lane: quality
status: done
priority: P1
size: M
tier: opus
depends_on: [T-5091]
spec: docs/specs/SPEC-0037-usage-tier-gates.md
---

## Goal
The daily progress dashboard and the steward's daily status lead with decision 0355's tier gates: foundational
and expert, with rates against their bars and the failing functions.

## Context
- SPEC-0037, design section 7. The report schema (`usage_gates`) and phase1_exit.py's tier lines are fixed in the
  spec. Build against a fixture until T-5092 lands, then run the dashboard on a real report.
- Code: tools/src/rustsheet_tools/progress_report/, tools/tests/test_progress_report.py,
  .claude/skills/progress-report/SKILL.md, .claude/skills/steward/SKILL.md, docs/specs/SPEC-0022.

## Acceptance criteria
- [x] A "Usage tiers" block follows the bottom line, with the tier rows, the top 10 failing functions per tier, and
  the change since the last dashboard. The Phase 1 checklist block shows the tier lines in place of Gate A and
  Gate B.
- [x] Fixture tests pin the block, including a report with no `usage_gates`, which shows "not measured".
- [x] The progress-report skill's judgments, and the steward's daily status roll-up, cite the tier line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Computing the gates (T-5092).

## Outcome
- New generated block `usage-tiers` (tools/src/rustsheet_tools/progress_report/usage.py) right after the bottom
  line: the tier gate line in bold, one row per tier (functions, cells, wrong type, outside, type and envelope
  rates against the pooled bars, the per-function bars, met), the `foundational` and `expert` lines of
  phase1-exit.json (read from `lines[]` by `criterion`, the existing line shape), the failing functions per tier
  (the report's `failing_functions`, sorted by cells missed, at most 10, then "and N more"), and the change against
  the last complete history row that has a `usage` object. A report without `usage_gates`, and a phase1-exit.json
  without the tier lines, both read "not measured".
- report.json `usage_gates` is read and checked in model.py (`load_usage_gates`, on `Parity.usage`). History rows
  gain an optional `usage` object, so the change follows the existing history.json delta pattern (like
  `codebase`), not a previous page.
- The parity block's Phase 1 gate list now leads with the tier gates. Gate B and Gate A follow, marked information
  (decision 0355 item 5) once the report measures the tiers, and still gating until then.
- `make progress-report` prints `usage tiers: <line>`. The steward skill makes that line the snapshot's first
  bullet; the progress-report skill's bottom line and read-and-compute rating cite the tier gates.
- SPEC-0022's layout table gains row 1a. Fixture tests in tools/tests/test_progress_report.py pin the block.
- Follow-up: once T-5092 lands, run the dashboard on a real report (see Context) and confirm its `usage_gates` and
  phase1-exit lines read unchanged. docs/progress.md was not regenerated here.
