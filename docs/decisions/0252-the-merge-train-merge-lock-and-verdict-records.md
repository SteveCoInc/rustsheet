# 0252 — The merge train, merge lock and verdict records are deleted; reviewer verdicts are an unchecked agent attestation

- **Status:** accepted (2026-11-02, T-3827). It amends 0247 and supersedes 0178, and retires
  0075, 0157 and 0169 together with their code; SPEC-0027 is marked superseded (kept as history).
- **Context:**
  - 0247 moved merges to GitHub's merge queue and retired the merge train (0157) and the merge
    lock (0075), but left their code in place so that the switch-over PR stayed small: about
    11.1k lines of `scripts/merge_*.py`, `scripts/review_panel.py` and their tests, fourteen
    Makefile targets, the `merge-train/*` push trigger of `ci.yml` (0147) and the train sections
    of the skills. Dead tooling misleads agents and costs CI time (the python job ran those tests
    on every PR).
  - The verdict records (0178) were the only machine check of reviewer verdicts, and only `make
    mark-ready` read them. The security review of T-3826 asked that they be deleted only with a
    replacement, or with this record saying outright that the verdicts go unchecked.
  - Four other scripts imported their git helpers from `merge_lock.py`, and `make catch-up` and
    `make renumber` (0207, 0224) built on `merge_ready.py`. Both targets still serve the queue:
    work-loop step 6.3 uses them for a PR the queue removed or that conflicts with `main`.
- **Decision:**
  - **Deleted:** `scripts/merge_lock.py`, `merge_ready.py`, `merge_train.py`,
    `merge_train_filing.py`, `merge_train_shadow.py`, `merge_train_stats.py`, `merge_step.py`,
    `merge_janitor.py`, `merge_flip.py`, `review_panel.py`, `merge_train.toml` and their eleven
    test files; the Makefile targets `merge-lock-acquire`, `merge-lock-wait`,
    `merge-lock-release`, `merge-lock-status`, `merge-ready-verify`, `mark-ready`,
    `verdict-record`, `merge-labels`, `janitor`, `merge-train`, `merge-train-ack`,
    `merge-train-next`, `merge-flip-check` and `merge-train-stats`; the lock-flip waiver of
    `check_governance.py` (0157); the carry rule (`merge_ready.Repo.carry_problem`,
    `id_renumber.renumbered_merge`); the `merge-train/current` and `merge-train/shadow` push
    triggers and the train branch case of `ci.yml`'s `changes` job (0147), whose pin in
    `test_check_ci_workflow.py` is now `["main"]`; `.claude/skills/review/SKILL.md` in
    `ci_changes.CODE_READ_PATHS` (its reader, `test_merge_ready.py`, is gone, so the skill is
    docs-only again).
  - **Kept, with the reason:**
    - `scripts/git_ops.py` (new) holds `git`, `GitError`, `merge_tree` and `push_lease` from
      `merge_lock.py`, the one definition that `id_registry.py` (its compare-and-swap push,
      0215), `id_renumber.py`, `new_case_owners.py`, `suite_meta.py`, `visible_lock.py` and
      `merge_catchup.py` share.
    - `make catch-up` and `make renumber` (`merge_catchup.py`, `merge_renumber.py`,
      `id_renumber.py`): the fetch of `main` and `checked_baselines` moved from
      `merge_ready.Repo` into `merge_catchup.py`. They now tell the agent to enqueue again once
      CI is green; `id_renumber.check_mapping` still checks every renumber before it is pushed.
    - `make merge-mode`, which now always prints `lock` (= "enqueue"): the routines' stored
      prompts (owner-managed; agents never edit routines) ask for it before enqueueing, and a
      missing target would make every worker refuse to enqueue. It goes when the owner drops it
      from the routines.
    - `scripts/gh_client.py`, which `make catch-up` uses; trimming the requests only the deleted
      scripts made is T-3866.
  - **Reviewer verdicts are an agent attestation that no machine checks.** No replacement for the
    verdict records is added: the merge queue's required checks are CI only, and an agent that
    could fabricate a verdict could equally fabricate a status check posted from its own session.
    The PR body's review table (work-loop step 6.2) is the record, a verdict counts only for the
    head its report's `HEAD:` line names (the `review` skill, step 3), and the steward's weekly
    spot-audit re-runs reviewers with the stronger model. This is the same trust 0053 had before
    the train existed.
- **Consequences:**
  - The python job runs about 9k fewer test lines; the review skill, steward and work-loop skills
    and `docs/process/` name no deleted target or mode.
  - **Owner steps:** delete the `ready-to-merge`, `merge-train-ejected`, `merge-train-stop` and
    `merge-hold` labels and the `merge-lock` and `merge-train/*` branches (the session git proxy
    refuses branch deletion, and agents have no label-delete tool); drop `make merge-mode` from
    the routine prompts when convenient. Nothing in the repository applies or reads those labels
    or branches any more.
  - Rollback: revert this record's PR; the train stays unusable without 0247's rollback too.
