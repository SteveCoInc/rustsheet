# 0247 — Merges go through the GitHub merge queue; the merge train and the merge lock are retired for merges

- **Status:** accepted (2026-11-02, owner request, T-3826). It reactivates 0053 and amends 0075
  and 0157. The repository moved from `steveash/rustsheet` (a personal account, which cannot turn
  the queue on: 0075's reason) to the `SteveCoInc/rustsheet` organization so that the queue can
  be used.
- **Context:**
  - On 2026-11-01/02 the merge train (0157) merged about 4.1–4.25 PRs an hour while PRs arrived
    at about 5.5 an hour. The ready queue grew from about 45 to 54 PRs, the ejected PRs from 6 to
    13, and the time from ready to merged reached 16–19 h.
  - Causes: the merger routine ran hourly (`35 * * * *`; routines.md said `5,35`); `collect()` in
    `scripts/merge_train.py` checks every ready PR one at a time while it holds the lock, which
    took about 19 min of the 40-min `HOLD_BUDGET` with 44 ready PRs; `BATCH_MAX` is 5; the
    LLM-driven step mode (0169) was fragile; and the long waits caused `conflict with main` and
    `eval red on main` ejections.
  - Shortening CI from about 10 to 6 min (T-3699, 0232) and raising the train's CI timeout to
    15 min (T-3488, 0242) did not raise throughput. Linear history and squash merges are not the
    bottleneck and stay.
  - GitHub's merge queue does what the train does (test each PR merged with current `main` and
    the PRs ahead of it, in groups, and drop a failing one) without a lock, a merger routine or
    an agent in the loop. `ci.yml` already runs on `merge_group` (0053).
- **Decision:**
  - **0053 is in force.** With CI green on the latest head and every applicable reviewer
    approving, the PR's agent (or the steward, for an orphaned PR) enqueues it with
    `enable_pr_auto_merge`, `mergeMethod: SQUASH`, and moves on. The queue gives 0028's
    guarantee, so the agents' up-to-date check (0028) is skipped and `main` is never merged in
    only because the branch is behind (0121 unchanged). A PR the queue removes is red CI. A
    queued PR gets no push: `disable_pr_auto_merge` first, push, re-run the reviewers the change
    concerns, enqueue again. Task-only PRs (0231) are enqueued once CI is green, without a panel.
  - **0075 is retired for merges.** No agent takes the merge lock to merge. The lock script
    stays until the train code is deleted (T-3827), but nothing in the skills uses it.
  - **0157 is amended: the train is retired.** `scripts/merge_train.toml` says `mode = "lock"`,
    and in `lock` mode the skills now mean "enqueue". `make mark-ready` and the
    `ready-to-merge` and `merge-train-ejected` labels are no longer used for new work; the merger
    routine is retired (routines.md). 0157's one-file flip to `lock` and its review waiver no
    longer serve a purpose, since the mode stays `lock`. A change to `shadow` or `train` would
    still be fully gated, and nothing may make one until the train code is gone.
  - **Owner settings** are 0053's: in the `main` ruleset, "Require merge queue" (Squash, build
    concurrency 5, group size 1–5, only merge non-failing PRs, 60-min check timeout) with the
    five required checks unchanged, "Require branches to be up to date" off, and "Allow
    auto-merge" on under Settings → General.
- **Consequences:**
  - A PR costs one CI run on its own head and one on its merge group, and up to five PRs share a
    group run. Ready-to-merged latency is bounded by queue CI, not by an hourly routine.
  - If `enable_pr_auto_merge` is refused because auto-merge or the queue is off, the agent leaves
    the PR green and approved, reports it under "Needs owner", and does not fall back to the lock
    or the train.
  - The open `ready-to-merge` PRs are enqueued oldest first after this record lands; the
    `merge-train-ejected` ones first take a rebase, `make check` and re-review (work-loop step
    6.3), then are enqueued.
  - Deleting the train code (`scripts/merge_*.py`, the janitor, step mode, SPEC-0027 and their
    Makefile targets and tests) is T-3827, as is dropping the verdict records (0178),
    which only `make mark-ready` reads.
  - Rollback: revert this record's PR; the routines would also need the merger back.
