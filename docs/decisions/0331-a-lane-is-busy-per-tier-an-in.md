# 0331 — A lane is busy per tier: an in-flight sonnet task no longer hides the lane's opus tasks

- **Status:** accepted (2026-11-07, T-3821). It amends 0022 (one task in flight per lane) and 0269
  (three engine slots): both now count per tier.
- **Context:**
  - On 2026-11-02 at 02:20 UTC every lane printed `no ready task` for the opus worker while the board
    held 25 ready, unclaimed opus engine tasks (the M2 exit owner T-3324 among them), 22 harness and
    28 research. Engine was busy only because of `agent/engine/T-3439-split-reference-operators`, a
    `tier: sonnet` P3 file split in another file. A sonnet task in flight hid every opus task of its
    lane, and the reverse, so one tier's worker idled behind the other's (decision 0018 runs one
    worker per tier).
  - 0022's reason for one task per lane is to keep parallel workers off each other's files. Measured
    over the last 7 days of `git log --first-parent origin/main` (2026-10-31 01:12 to 2026-11-07
    01:12 UTC; 994 squash-merged task PRs whose subject names a task and its PR): taking each PR's
    span as GitHub's PR creation to merge time (an underestimate of the branch's life, which starts
    at the claim push), **1,445 pairs of PRs in one lane overlapped in time, and 802 of them touched
    a common non-task file**. 437 of those shared files are `eval/baseline/probes.txt`; most of the
    rest are `docs/excel-semantics/*.md`, append-mostly files the queue merges cleanly. Counting only
    files outside `docs/` and `eval/baseline/`, **231 overlapping pairs shared a code file**.
    By tier (the tier on `main` today, so a handed-off task counts as opus): **466 of the overlapping
    pairs were one sonnet and one opus task; 275 of those shared a non-task file and 36 shared a code
    file**. Same-tier pairs: 979 overlapping, 527 sharing a non-task file, 195 a code file. So the
    lanes already ran tasks side by side (the busy rule sees only the last 2 h of commits, while PRs
    wait in review and the queue longer), and a cross-tier pair shares code less often (36 of 466,
    7.7%) than a same-tier pair (195 of 979, 19.9%).
  - The merge queue (0247) tests each PR against the PRs ahead of it and drops a conflicting one;
    its session or an adopter (0248, 0253) resolves it and enqueues it again.
- **Decision:** `scripts/tasks.py` `available()` counts slots per (lane, tier): a lane is busy for a
  tier once it has as many in-flight tasks of that tier as `LANE_SLOTS` gives the lane (one; three
  for `engine`), so a lane holds at most one task per tier in flight (three per tier in engine). A
  branch's tier is its own task file's: `opus` once it has a `## Handoff` heading (resumed or not) or
  the step-5c `Resumed after the review-round limit` marker, else its `tier:`; with no tier on the
  branch, `main`'s effective tier; and a branch whose tier is unknown takes a slot of both tiers.
  `next`, `ready` and the `lane L busy` message (0330) all apply it, and the message names only
  branches that hide the asking tier's tasks. Claims, handoffs, re-reviews and the P0 bypass (0330)
  are unchanged.
- **Consequences:**
  - The opus worker no longer finds a critical lane empty only because a sonnet task is in flight
    there, and stops filing planner tasks into queues it cannot see.
  - A lane may have one more task in flight; by the count above, about one cross-tier pair in 13
    touches a common code file, and the queue drops and the adopters resolve those.
  - The worker prompts in `docs/process/routines.md` describe the per-tier rule; the owner re-applies
    the routines (`docs/status.md`, "Needs owner").
  - Revisit if queue drops for conflicts between same-lane PRs of different tiers grow: then split
    the lane, not the rule.
