# 0253 — A PR the merge queue drops is labelled queue-dropped and adopted at once

- **Status:** accepted (2026-11-02, T-3831). It amends 0248 (orphaned agent PRs).
- **Context:** Under 0248 an agent PR is an orphan once its branch has been idle 4 h, and the opus
  worker adopts one orphan per run (work-loop step 0.3). On 2026-11-02 the merge queue removed 18
  of 49 enqueued PRs within about an hour, as PRs ahead of them merged and conflicted with them;
  at one adoption per 2-hour run that is a day of delay. The 4 h wait keeps an adopter from racing
  a live session, but a session that enqueued a PR has moved on (work-loop step 6.3), so a drop
  needs no such wait. The session proxy refuses GraphQL, which has the merge-queue timeline events, and
  the REST issue timeline does not document them, so finding drops from an agent session is not
  cheap; GitHub Actions does get them, as the `pull_request` activity types `enqueued` and `dequeued`.
- **Decision:**
  - `.github/workflows/queue-drops.yml` labels an open PR from this repository `queue-dropped`
    when it is dequeued, and removes the label when it is enqueued, its branch moves
    (`synchronize`) or it closes. A labelled PR is therefore one the queue dropped that nobody has
    pushed to since. The job checks out nothing and runs no PR code; only it holds write rights
    (`issues`, `pull-requests`), the workflow level stays `contents: read`.
  - A labelled PR is an orphan at once (no 4 h wait). The opus worker adopts every labelled PR at
    the start of its run, oldest first, while its session budget (0229) allows new tasks, then
    the oldest other orphan under the 4 h rule, one per run. Each adoption counts as one task.
    The steward adopts them the same way in its daily run.
  - An agent changing its own queued PR commits the change first, then disables auto-merge and
    pushes at once, so its PR carries the label only for seconds (work-loop step 6.3).
- **Consequences:**
  - Dropped PRs are picked up within one opus run (about 2 h) instead of one per run.
  - A merge by GitHub may also fire `dequeued`; the label is then on a closed PR, which no rule
    reads (orphans are open), and the `closed` event clears it.
  - Owner step: update the opus worker routine's step (2b) to the text in
    `docs/process/routines.md`. Until then the routine's old text still adopts dropped PRs after
    4 h; the work-loop skill already has the new rule.
  - Rollback: delete the workflow and revert the skill text; the 4 h rule of 0248 applies again.
