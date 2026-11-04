# 0264 — Steward triages stale draft PRs

- **Status:** accepted (2026-11-04, owner request, T-4046). It amends 0048 (tasks blocked on their
  branch), 0050 and 0248 (orphaned PRs) and 0013 (unblocking is an agent's job).
- **Context:**
  - On 2026-11-04 nine draft PRs (#1885, #2529, #2573, #2624, #2635, #2640, #2671, #2717, #2788)
    had been drafts for 2 to 9 days. The owner had noticed the same set two days earlier. No
    routine looked at them:
    - the orphan rule (0050, 0248; steward skill "Orphaned agent PRs", routine step 5, work-loop
      step 0.3) only adopts PRs that are *not* drafts;
    - a task blocked on its branch is claimed at any age (0048, `scripts/tasks.py`
      `branch_claims`), so `make next-task` never offers it again, and the opus worker's "resume an
      idle branch" step skips blocked tasks;
    - 0013 makes unblocking the steward's job, but the steward reads `blocked` from `main`'s task
      files, and these tasks say `todo` there: the blocker lives only on the branch.
  - So a PR became a draft for a good local reason (blocked, partial, task-only, reviews pending)
    and then nothing looked at it again. What that cost, by PR:
    - #2635 duplicated work that had merged as #2809;
    - #2671 waited on T-3579, which had been `done` for days (#2728);
    - #2624 now collides with two task ids on `main`;
    - #2788's "Needs owner" entry lived only on the draft branch, so the owner was never asked
      for the ruling it needs;
    - #1885's financial.md quirk entry never reached `main`, against CLAUDE.md directive 6.
- **Options considered:**
  - *Auto-close drafts after N days.* This loses the oracle evidence, probes and quirk notes that
    many of them hold, and it says nothing to the owner when a ruling is what blocks them.
  - *Forbid drafts.* Drafts are useful: CI runs on them while reviewers run (work-loop step 5),
    and a blocked branch keeps its evidence.
  - *Give drafts an age limit in `branch_claims`.* This frees the task but leaves the PR, its
    evidence and its owner question where nobody looks. It could come later as tooling (out of
    scope here).
- **Decision:** the daily steward run triages every **stale draft**: an open draft PR from an
  `agent/*` or `claude/*` branch of this repository whose head has not moved for 24 h. The steward
  puts each one in exactly one bucket and acts on it, as the steward skill's "Stale drafts" rule
  lists:
  - superseded: close it;
  - blocker gone: unblock it on its branch;
  - owner needed: make sure `docs/status.md` asks the owner;
  - actually ready: mark it ready for review, so the orphan rule adopts it;
  - otherwise: salvage its evidence through a filed task, or comment on what still blocks it.
  Every stale draft is listed with its age and bucket in the steward PR, so a draft can no longer
  sit unseen.
- **Consequences:**
  - The steward skill gains a "Stale drafts" rule, and routine step 5 in docs/process/routines.md
    names it. The stored steward routines need the owner to re-apply that prompt; until then the
    steward still reads the skill, which the stored prompt already points it to.
  - Closing a draft deletes nothing: the branch and its commits stay, and the closing comment
    links what replaced it.
  - The steward never pushes code to a stale draft. It makes bookkeeping commits only:
    unblocking a task on its branch (0048, 0013) or a clean merge of `main` to fix a conflict
    (0121). Salvage (moving probes, quirk entries and notes onto a fresh branch from `main`) is a
    filed task, so the steward run stays bounded.
  - A draft whose head was pushed by a live session within 24 h is never touched (same reason as
    the 2 h and 4 h waits in 0050 and 0248).
