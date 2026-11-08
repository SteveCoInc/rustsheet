---
id: T-5095
title: "Task usage tiers: usage key, title inference, foundational-first ordering, new --usage (SPEC-0037 step 5)"
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
When priority is otherwise equal, routines pick tasks about foundational functions before those about expert
functions (decision 0355 item 7), and new tasks record their function's tier.

## Context
- SPEC-0037, design section 8.
- Code: scripts/tasks.py (`available` sort key, `new`, validation), scripts/test_tasks.py,
  docs/process/task-format.md, .claude/agents/planner.md, .claude/agents/gap-triager.md,
  .claude/skills/work-loop/SKILL.md.
- The tiers come from eval/catalog/scope.json's `usage` (T-5091).

## Acceptance criteria
- [x] The optional `usage: foundational | expert` key is validated and documented.
- [x] `effective_usage` uses the key, then the title, then none, and its tests cover each.
- [x] `available()` orders foundational first right after priority, and a test shows two tasks of the same
  milestone and priority ordered by tier.
- [x] `next` and `ready` show `usage: foundational`. `new --usage`, or the tier inferred from the title, writes
  the key.
- [x] The planner and gap-triager definitions and the work-loop skill say so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-tagging existing task files in bulk (inference covers them).

## Outcome
- The tier logic is a new stdlib module, scripts/task_usage.py (`USAGES`, `catalog_tiers`, `title_usage`,
  `effective_usage(task, root)`, `usage_note`), beside milestone_order.py and pivot_owners.py: tasks.py was 997
  lines against the 1000-line file-size ratchet. It now has 998: the `campaign`/`defer`/`usage` checks share one
  loop, and two blocks in `branch_claims` and `stats` lost redundant lines (no behavior change).
- `validate` rejects a `usage:` other than `foundational` or `expert`. `available()` sorts on
  `effective_usage(t) != "foundational"` right after priority. `next` and `ready` append `  usage: foundational` to
  a foundational task's line (after the path on `next`, before any `resume` note). `new --usage
  foundational|expert` writes the key, and without the flag the tier the title names, if any;
  `make new-task USAGE=...` passes it through.
- Title inference reads eval/catalog/scope.json once per tree (`functools.cache`) and takes the tier of the
  first catalog name in the title, left to right (coordinator follow-up; SPEC-0037 design 8 item 2 now says
  so): a title names its subject first. A name counts when it is
  upper case, not the tail of a longer name, a hyphenated word, a sheet or absolute reference (`P!SUM`,
  `$SUM`) or an error literal (`#N/A`), and followed by `(`, `,`, a space or the end. Short-name rule: `N`,
  `T`, `NA`, `TRUE` and `FALSE` count only before `(` (titles use them as a letter, `Top-N`, a population
  size `N 6`, a value `a TRUE cell`, or "not available"). Other short names such as IF, AND, OR, NOT and DAY
  count as words: matching is case-sensitive, and in every current title their all-caps form is the function.
- Open tasks (850) at this commit infer foundational 78, expert 361, none 411 (ready: 65 / 189 / 309 of
  563). No task carries the key yet.
- First-name inference fixes the wrong tiers "any foundational name" gave titles that quote a formula: T-3061 (`T.INV is
  SQRT(...)`) and T-3242 (`GAMMA.DIST ... EXP(...)`) now read expert, from their subject function. Where a
  title's first function is not the task's, the `usage:` key overrides (task-format.md). Bulk re-tagging is
  out of scope.
- Tests: scripts/test_task_usage.py (19 tests: first-name inference, a formula quoting a foundational
  function after the subject, the short-name rule, the cache, the real catalog,
  `effective_usage`, validation, ordering at equal and unequal priority, `next`/`ready` lines, `new --usage`).
