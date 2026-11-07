# 0332 — A current-milestone task passes a lane busy only with later-milestone work

- **Status:** accepted (2026-11-07, T-3824). It amends 0022 (one task in flight per lane), as 0331
  (per tier) and 0269 (three engine slots) left it.
- **Context:**
  - On 2026-11-02 the research lane was busy with `agent/research/T-3571-bfrac-fraction-arithmetic`,
    an M9 opus task. That hid T-3584 (M2, opus, ready), the probe that T-3445 waits on, and T-3445
    owned 55 of M2's 264 remaining exit cells. The engine lane showed the same pattern whenever its
    in-flight branch was an M7-M9 task (T-3382 on 2026-11-01). Each such wait delays the current
    milestone's exit by the later branch's whole run.
  - 0022 keeps parallel workers off each other's files. A current-milestone task and a later-
    milestone task in the same lane usually touch different modules (T-3584 is a case-mapping
    probe, T-3571 fraction arithmetic). The merge queue (0247) drops a PR that conflicts with one
    ahead of it, and its session or an adopter (0248, 0253) resolves it. Decision 0331's measure
    applies here too: 231 of the 1,445 same-lane PR pairs that overlapped in time shared a code file.
- **Decision:** in `scripts/tasks.py` `available()`, a ready task whose milestone is the current
  milestone is offered in a lane that is busy for its tier when every in-flight branch filling that
  lane and tier's slots works a task of a later milestone in the phase order. The inputs each have
  one definition, read by `scripts/milestone_order.py`:
  - the current milestone is the `(M<n> is current)` of the `## Snapshot` heading in
    `docs/status.md`, which the steward rewrites every run. Without one, the rule is off;
  - the phase order is the table under `## Phase order` in `docs/milestones.md` (decision 0027),
    in row order. M0 comes first, so an M0 branch is never passed, and a branch whose task is not on
    `main`, or whose milestone the table does not list, is never passed either.
  Once a current-milestone task is in flight there, the lane holds a branch that is not later, so
  no second task passes it: at most one pass per lane and tier. `next`, `ready` and the
  `lane L busy` message (0330) all apply the rule; the message is printed only when no task passes.
  P0 (0330), claims, handoffs and re-reviews are unchanged. Candidates are still sorted by milestone
  number, not by phase order (out of scope for T-3824).
- **Consequences:**
  - The current milestone's exit work no longer waits for an unrelated later-milestone task in its
    lane to merge.
  - A lane may hold one more branch per tier. A conflict between the two goes through the queue's
    drop and the adopters, as under 0331.
  - The worker prompts in `docs/process/routines.md` describe the rule; the owner re-applies the
    routines (`docs/status.md`, "Needs owner", extended from 0331's entry).
