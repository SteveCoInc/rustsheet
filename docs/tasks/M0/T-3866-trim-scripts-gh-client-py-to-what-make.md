---
id: T-3866
title: "Trim scripts/gh_client.py to what make catch-up uses"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-3827]
---

## Goal
`scripts/gh_client.py` holds only the GitHub requests a script of this repository still makes, so
no dead request builders or tests are left from the deleted merge train.

## Context
T-3827 (decision 0252) deleted the merge train, the merge lock and the readiness handoff, which
made most of `gh_client.Client`'s requests. After it, only `scripts/merge_catchup.py` (`make
catch-up`, and `make renumber` through it) imports `gh_client`, for `from_remote`, `number`,
`branch_name`, `Client.pull`, `GhError`, `field_of` and `text_of`. Methods such as `activity`,
`add_label`, `create_label`, `remove_label`, `closed_labelled`, `create_pull`, `delete_branch`,
`dispatch`, `workflow_runs`, `run_jobs`, `check_runs`, `pull_files`, `pull_commits`, `pulls`,
`updated_pulls`, `pr_finished`, `comments`, `reviews`, `permission`, `comment`, `label`, `commit`,
`merge`, and the helpers `label_names`, `int_of`, `iso`, `when`, have no caller outside
`scripts/test_gh_client.py` (check with `rg -w <name> scripts tools`). The docstring of
`gh_client.py` carries `TODO(T-3866)`.

## Acceptance criteria
- [x] Every public function, method and constant of `gh_client.py` has a caller outside its own
      test, or is a part of the request machinery (`Request`, `send`, `call`, `GhApi`, `FakeGh`)
      those callers need; the rest and their tests are deleted.
- [x] `scripts/test_gh_client.py` still checks that the fake and `gh api` build the same requests
      for every method kept, and that no caller text reaches another API path.
- [x] The `TODO(T-3866)` in the module docstring is gone; `scripts/check_duplication.py --update`
      if it shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
- `make catch-up` and `make renumber` behavior.

## Outcome
`scripts/gh_client.py` shrank from 484 to ~190 lines: only `Request`, the validators `number`,
`branch_name`, `checked`, `as_dict`, `Client` (`call`, `pull`), `GhApi`, `FakeGh`, `Reply`,
`from_remote`, `owner_repo`, `field_of`, `text_of` and `GhError` remain (what `merge_catchup.py`
imports). All request builders, paging, cursor, label and time helpers and their tests were deleted;
`TODO(T-3866)` is gone. `test_gh_client.py` keeps the fake-vs-`gh api` same-request check and the
refusal checks for what remains. Eval delta: none (scripts only). `make check-fast` green;
`check_duplication.py --update` left the count unchanged.
