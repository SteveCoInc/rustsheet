---
id: T-5095
title: "Task usage tiers: usage key, title inference, foundational-first ordering, new --usage (SPEC-0037 step 5)"
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
When priority is otherwise equal, routines pick tasks about foundational functions before those about expert
functions (decision 0355 item 7), and new tasks record their function's tier.

## Context
- SPEC-0037, design section 8.
- Code: scripts/tasks.py (`available` sort key, `new`, validation), scripts/test_tasks.py,
  docs/process/task-format.md, .claude/agents/planner.md, .claude/agents/gap-triager.md,
  .claude/skills/work-loop/SKILL.md.
- The tiers come from eval/catalog/scope.json's `usage` (T-5091).

## Acceptance criteria
- [ ] The optional `usage: foundational | expert` key is validated and documented.
- [ ] `effective_usage` uses the key, then the title, then none, and its tests cover each.
- [ ] `available()` orders foundational first right after priority, and a test shows two tasks of the same
  milestone and priority ordered by tier.
- [ ] `next` and `ready` show `usage: foundational`. `new --usage`, or the tier inferred from the title, writes
  the key.
- [ ] The planner and gap-triager definitions and the work-loop skill say so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-tagging existing task files in bulk (inference covers them).
