---
name: review
description: Run RustSheet's multi-agent code review on the current branch versus main - design (architecture and quality), security on executable changes, Excel fidelity reviewers (performance is off, decision 0081; panel by diff kind, decision 0236) in parallel - and consolidate verdicts. Use before opening or merging any PR.
---
# Multi-agent review

A **task-only PR** gets no review: it is enqueued once CI is green, and the merge queue's `tasks.py
validate` on the combined tree is its gate (decisions 0231, 0247). Task-only means every changed
path is a task file: a regular file (mode `100644`, never a symlink, `120000` in `git diff --raw
origin/main...HEAD`) under `docs/tasks/` whose name matches `tasks.TASK_FILE` (`T-NNNN-<slug>.md`).
So `docs/tasks/README.md`, any other `.md` there, or a symlink brings the panel of step 3 back, as
does any path elsewhere.

1. Make sure the branch is committed and `make check-fast` passed (reviewers are not linters).
2. Collect context: `git diff --stat origin/main...HEAD`, the task id (branch name) and its file.
3. Launch these subagents **in parallel** (one message, multiple Agent calls), each with a
   prompt containing: the task id, task file path, spec path(s), and "review `git diff
   origin/main...HEAD`":
   - `reviewer-design` (always): architecture and code quality, one reviewer (decision 0236)
   - `reviewer-security` (unless every path is under docs or eval/suites; docs/process keeps it):
     a diff of only task files, decisions, specs, semantics notes and suite cases has nothing
     executable to review (decision 0236). Docs that code reads (`ci_changes.CODE_READ_PATHS`:
     docs/excel-semantics, docs/progress, docs/milestones.md) keep it too. Any path outside those,
     or a list you are unsure of, gets it.
   - `reviewer-excel-fidelity` (if the diff touches crates/rustsheet-engine, crates/rustsheet-xlsx,
     eval/suites, or eval/baseline)
   - Not `reviewer-performance`: it is off while correctness comes first (decision 0081). A bench
     criterion in the task is met at no more than 50% slower than main.
   Each reviewer runs on the model set in its agent frontmatter (decision 0015: security and
   design on Sonnet; Excel fidelity on Opus). Do not override it
   per call, with one exception: the steward's weekly spot-audit launches every reviewer with
   `model: opus`, so a cheaper reviewer's miss is re-checked by the stronger model.
   A verdict counts only for the commit its report's `HEAD:` line names: if HEAD has moved past
   it, or the report has no `HEAD:` line, that reviewer runs again on the current head, and a
   re-run on the same head replaces the earlier verdict. The panel's verdicts are an agent
   attestation that no machine checks (the head-bound verdict records of decision 0178 went with
   the merge train; decision 0252): the table of step 6 is the record, so report each verdict in it
   exactly as the reviewer returned it.
4. Consolidate into a table: reviewer | verdict | blocking findings. A PR is mergeable only when
   every reviewer returned `APPROVE` (MINOR findings may be deferred as follow-up tasks).
   **Push once per round** (decision 0240): wait until every reviewer of the round has reported,
   fix the round's blocking findings (and any MINOR ones you choose to fix) in local commits, run
   `make check-fast`, then push once and start the next round on that head, where a reviewer whose
   approval names an older head runs again (step 3). Do not push while a reviewer of the round is
   still running, and do not push fix by fix: each push starts a full CI run and cancels the one
   in flight, and a reviewer still running is then reviewing a head that no longer counts. Red-CI,
   conflict and queue-removal fixes are not review-round fixes: push them when ready (steward skill).
5. For each MINOR finding you choose not to fix, create a follow-up task or note why it is moot.
6. Paste the table into the PR body ("Agent review" section).

Reviewers must not see your reasoning about why the code is fine — give them only the task
and the diff, so their review is independent.
