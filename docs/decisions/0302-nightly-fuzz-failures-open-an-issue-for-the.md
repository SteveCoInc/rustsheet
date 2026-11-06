# 0302 — Nightly fuzz failures open an issue for the issue intake

- **Status:** accepted (2026-11-06, T-4490; the owner approved this route in the session that filed it)
- **Context:** Decision 0035 made the steward file one task per distinct fuzz crash. Nothing told
  it that a run was red, though:
  - its daily routine prompt (docs/process/routines.md) has no step that reads scheduled
    workflow runs;
  - a scheduled run on `main` wakes no agent, unlike a PR's checks.

  `bitcode_workbook` then failed on 2026-11-02 and 2026-11-03 (runs 41384443922 and
  41573652529). Both were timeouts, and no task was filed until the owner noticed (T-4048).
  Two other routes were considered:
  - A daily "fuzz sweep" step in the steward. It reacts a day late, and the stored routine
    prompt would need an owner edit before it took effect.
  - Keeping fuzz results only in the Actions UI. That is the status quo that failed.
- **Decision:** `.github/workflows/fuzz.yml` gets a `report` job that runs only after a failed
  scheduled run on `main`.
  - **What it does:** for each failed `fuzz (<target>)` job it opens one issue titled
    `Nightly fuzz: <target> failed`, labelled `fuzz-failure`. If that target already has an open
    issue, it comments on it instead.
  - **What the issue holds:** the run link, and each saved input's kind, size and SHA-1, with its
    base64 embedded while the body stays under GitHub's limit (`scripts/fuzz_issue.py`).
  - **Permissions:** only the `report` job gets `issues: write` and `actions: read`, to list the
    run's jobs and create the label and issues. The fuzz jobs keep `contents: read`, no secrets and
    no `pull_request_target`. The job runs this repository's script on the run's own job names and
    artifacts. Job names are matched against `fuzz \(<target>\)`, and target names against
    `[a-z][a-z0-9_]*`, before any of them reaches a command.
  - **Who works it:** the issue intake routine, triggered on `issue.opened`. Its skill gains a
    "Fuzz finding" kind: P0, the reproducer's bytes treated as data, exempt from the per-author
    limit. It applies only when the author is `github-actions[bot]`; the same title or label from
    anyone else is an ordinary issue.
  - **Backstop:** the steward's daily issue sweep catches an issue whose trigger was dropped. The
    steward files the task itself only when a red run has no issue at all, because its `report`
    job failed.
- **Consequences:**
  - A red nightly fuzz run becomes a task within minutes, with no owner step and no routine
    change.
  - Timeouts and out-of-memory failures are named as findings alongside crashes (steward skill,
    agent-workflow.md).
  - **One issue per target while it is open:** a second, different failure of the same target
    arrives as a comment, and the intake does not re-read comments. It is worked from that issue
    (as a follow-up with `issue_origin:`), or it fails again and opens a new issue once the first
    fix has closed the old one.
  - **Fork pull requests:** they cannot reach the `report` job, which needs a scheduled run on
    `main`.
  - **Unverified:** that the intake routine's GitHub trigger fires for an issue opened with the
    workflow token (Actions does not start workflows from `GITHUB_TOKEN` events; app webhooks are
    still sent). If it does not, the steward's daily sweep picks the issue up, as with any dropped
    trigger.
