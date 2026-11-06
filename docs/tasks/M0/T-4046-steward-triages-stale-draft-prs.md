---
id: T-4046
title: "Steward triages stale draft PRs"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
No draft PR from an agent branch sits unseen for days. Each day the steward closes it, unblocks it,
asks the owner, readies it or salvages its evidence.

## Context
On 2026-11-04 the owner reported nine drafts unchanged for over 24 h, the same set as two days
earlier: #1885, #2529, #2573, #2624, #2635, #2640, #2671, #2717, #2788. Nothing had picked them
up, for three reasons:
- the orphan rule (decisions 0050, 0248) skips drafts;
- a task blocked on its branch stays claimed at any age (decision 0048, `scripts/tasks.py`
  `branch_claims`);
- the steward's unblock check (decision 0013) reads `main`, where these tasks still say `todo`.

What that cost, by PR:
- #2635 duplicated #2809;
- #2671's blocker (T-3579) had been `done` for days;
- #2788's request for an owner ruling never reached `docs/status.md` on `main`;
- #1885's financial.md quirk entry never reached `main`.

Decision 0264 has the full account and the options considered.

## Acceptance criteria
- [x] Decision 0264 records the rule.
- [x] The steward skill has a "Stale drafts" rule. It defines a stale draft (draft, `agent/*` or
  `claude/*` branch of this repository, head idle 24 h) and gives five ordered buckets
  (superseded, blocker gone, owner needed, ready, otherwise) with the action for each.
- [x] Routine step 5 in docs/process/routines.md names the rule. Agent-workflow's "Unblocking is
  an agent's job" points at it.
- [x] The rule never pushes code to a draft, never deletes a branch, and never touches a draft
  pushed in the last 24 h.
- [x] `make check-fast` green; reviewers approve.

## Out of scope
- Re-applying the stored steward routine prompts (only the owner can edit routines).
- An age limit on branch claims in `scripts/tasks.py` (decision 0264, options considered).
- Re-running the triage: the owner approved applying it by hand to the nine drafts in this task's session
  (Outcome); later drafts are the steward's.

## Outcome
Docs and skill only: decision 0264, the steward skill's "Stale drafts" rule, routine step 5 and
one sentence in agent-workflow. No code or eval change. On 2026-11-04 the owner approved applying the rule to the
nine drafts by hand:
- closed as superseded: #2635 (merged as #2809); #2624 (task ids now collide with `main`); #2640 (decision 0250,
  T-3623 #2841, fixed the exit_owners parsing that it worked around);
- closed at the owner's request: #2788 (another session owns T-3544);
- blocker gone: #2671, with T-3424 unblocked on its branch (T-3579 is done) for the opus worker;
- ready: #2717, but the owner's other session had already closed it six minutes earlier and re-filed its
  task as T-4047 (#3114), so the mark-ready call failed on a closed PR. It was not a permission
  problem. Bucket 4's "refused" fallback (0a4a883fa) stays as a general safeguard;
- salvage filed, then closed: #1885 (T-4050), #2529 (T-4051; T-3406 refuted its engine stage), #2573 (T-4052).
