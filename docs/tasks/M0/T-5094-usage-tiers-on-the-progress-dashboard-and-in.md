---
id: T-5094
title: "Usage tiers on the progress dashboard and in the steward's daily status (SPEC-0037 step 4)"
milestone: M0
lane: quality
status: in-progress
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
- [ ] A "Usage tiers" block follows the bottom line, with the tier rows, the top 10 failing functions per tier, and
  the change since the last dashboard. The Phase 1 checklist block shows the tier lines in place of Gate A and
  Gate B.
- [ ] Fixture tests pin the block, including a report with no `usage_gates`, which shows "not measured".
- [ ] The progress-report skill's judgments, and the steward's daily status roll-up, cite the tier line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Computing the gates (T-5092).
