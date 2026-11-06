# 0306 — Owner ruling: work M15 tasks before Phase 1 exits, one at a time, outside the routines

- **Status:** accepted (2026-11-06, T-4167)
- **Context:**
  - Decision 0275 created M15 (xlsx round-trip fidelity) in Phase 2. It filed its tasks
    `blocked` until Phase 1 exits, with no Phase 1 carve-out (D2).
  - On 2026-11-06 the owner asked an interactive session to work the M15 tasks now, one at a
    time, through the normal work loop, merging each before starting the next.
- **Decision:**
  - The owner-directed session may take M15 tasks before Phase 1 exits, in dependency order.
  - It unblocks only the task it claims (`blocked` → `in-progress` in the claim commit). Every
    other M15 task stays `blocked`, so `make next-task` never offers M15 work to the scheduled
    routines, and Phase 1 keeps their whole capacity.
  - Each task otherwise follows the work loop unchanged: branch, tests or a spec, `make
    check-fast`, the review panel, the merge queue.
  - Decision 0275's D2 still holds for the routines.
- **Consequences:**
  - M15 tasks can land before Phase 1 exits, and the milestone stays in Phase 2's order.
  - When Phase 1 exits, the steward unblocks whatever M15 tasks are left, as decision 0275 says.
  - This ruling ends with the owner's instruction. A later session that has not been told to
    work M15 leaves the tasks blocked.
