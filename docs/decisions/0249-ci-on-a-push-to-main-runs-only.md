# 0249 — CI on a push to main runs only the eval report; the merge group already ran the gates

- **Status:** accepted (2026-11-02, owner ruling, T-3704). It amends 0054 (the report on `main`
  is never cancelled) and 0147 (`main` is no longer one of the refs that run every gate), and
  settles T-3704's open question from SPEC-0027 section 5 for the merge queue.
- **Context:**
  - Under the merge queue (0053, 0247), every commit that reaches `main` is exactly the commit its
    merge group tested. On #2835, `main` moved to 7d63c17, the commit built and tested on
    `gh-readonly-queue/main/pr-2835-...`. The push run on `main` then repeated every gate on a
    tree that had just passed them.
  - Those repeat runs compete for runners with merge groups and PR CI. A free organization gets
    about 20 concurrent jobs, a CI run is about 7, and the queue builds up to 5 groups at once.
  - On 2026-11-01, 12 of 16 runs on `main` were cancelled by the next merge's run (shared
    concurrency group `ci-refs/heads/main`), so 0054's per-merge eval report mostly did not exist.
  - T-3704 (#2739) proposed to stop cancelling, under the train. Review blocked it on SPEC-0027
    section 5, which cancelled on purpose, and the owner ruled for the queue: run only the report.
- **Decision:**
  - The `changes` job outputs `gates`, which is `'false'` exactly on a push to `main`. Then
    rust, python and hygiene are skipped, the eval job skips `make eval-gates` and runs only the
    full report (`make eval`) and its artifact, and `rust (fmt, clippy, test)` passes over the
    skipped shards. A `workflow_dispatch` on `main` still runs every gate.
  - A push to `main` runs in a concurrency group of its own (`ci-refs/heads/main-<sha>`) with no
    cancelling, so every merge gets its report. Every other ref keeps one run per ref, and a newer
    push cancels the older one.
- **Consequences:**
  - A merge costs one report job on `main` (about 4 min) instead of seven gate jobs, and no
    `main` run is cancelled.
  - What still covers `main`: the merge group's full CI on the same commit, the nightly full eval
    (`nightly.yml`), and the steward's daily `make eval`. A push that bypasses the queue (only
    the owner can, through the ruleset's bypass) gets no gates on `main`. That is the owner's
    own act, and the next PR's CI and merge group run every gate on top of it.
  - The merge-train branches keep their trigger until T-3827 removes it.
  - Rollback: revert this record's PR.
