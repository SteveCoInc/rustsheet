# 0248 — Land PRs faster: a fast local gate, one CI run per PR, and orphans adopted within hours

- **Status:** accepted (2026-11-02, owner request, T-3828). It amends 0050 (orphaned PRs), 0229
  (the session budget it serves) and the work-loop steps of 0247.
- **Context:**
  - With the merge queue (0247), each PR runs the full gates at least three times: the worker's
    local `make check` (about 25 min in a session container, the whole Rust suite on one
    machine), PR CI on every pushed head (about 6 min, the suite in three shards), and the
    merge-group run.
  - The work-loop marks the task `done` only after review, so almost every PR pushes a second
    head and runs CI twice. T-3826's PR did: 07370e6 was reviewed, then 6c6e99b added the Outcome
    and MINOR wording fixes.
  - The long local run spends the 110-minute session budget (0229), and a session that ends
    before its PR is enqueued leaves it orphaned. Orphans are adopted only by the steward's daily
    run and only after 24 h open (0050). Since 0247, `claude/*` branches are not covered at all,
    and a PR the merge queue removes after its session ended waits for that daily run too.
- **Decision:**
  - **A fast local gate.** `make check-fast` (`scripts/fast_gates.py`) runs the gates of `make
    check` that the change can break: always `fmt-check`, `hygiene` and `typos`; `clippy` and
    `eval-gates` for Rust, eval data, ratchet files or docs that code reads; `py-check` for
    `tools/`; `py-bindings` for `bindings/`; all of them for a path it does not know or when the
    changed paths cannot be listed. It never runs the full Rust suite: workers run the tests of
    the code they changed (`cargo test -p <crate> <filter>`) and leave the suite to CI. The
    required CI checks, the merge-group run and `make check` are unchanged; no gate is removed,
    only its local duplicate. CI failures are still reproduced locally with the failing target.
  - **One CI run per PR.** The work-loop writes `status: done` and the Outcome in the commit that
    goes to review. Once every reviewer approves, a MINOR finding never starts a push: it becomes
    a follow-up task or a note on the PR, or rides along with a push made for another reason.
  - **Orphans within hours.** An orphan is an open, non-draft PR on an `agent/*` or `claude/*`
    branch, not queued (auto-merge off), whose branch has not moved for 4 h, including a PR the
    queue removed. The opus worker adopts the oldest orphan at the start of each run (work-loop
    step 0, counted as a task), and the steward adopts orphans in its daily run. Either one
    follows the steward skill's "Orphaned agent PRs" conditions: fix, re-review with its own
    panel, and enqueue. The 24 h minimum is dropped; the 2 h rule for closing duplicates stays.
- **Consequences:**
  - A worker saves about 20 minutes per task, and a docs or process change checks in about two.
    A Rust change that breaks a test elsewhere is caught by CI on the PR rather than before the
    push: one more CI round in that case, against minutes saved on every task.
  - CLAUDE.md's definition of done reads "`make check-fast` locally; every required check green
    in CI". Directive 2 (`make check` must pass) holds: CI and the merge queue run all of it.
  - An orphan waits at most about 4 h plus the time to the next opus worker run (2 h), not a day.
  - Rollback: revert this record's PR.
