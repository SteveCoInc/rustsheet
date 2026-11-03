# 0254 — Trim gh_client.py to what make catch-up uses

- **Status:** accepted (2026-11-02, T-3866). It follows 0252.
- **Context:** 0252 deleted the merge train, the merge lock and the readiness handoff, which made
  most of `scripts/gh_client.py`'s GitHub requests. Afterwards only `scripts/merge_catchup.py`
  (`make catch-up`, 0207, and `make renumber` through it, 0224) imports `gh_client`, for
  `from_remote`, `number`, `branch_name`, `Client.pull`, `GhError`, `field_of` and `text_of`. The
  other request builders (labels, comments, merges, workflow runs, check runs, paging and time
  helpers) had no caller outside `scripts/test_gh_client.py`. `gh_client.py` and its test are
  protected paths, so the trim needs this record.
- **Decision:** `gh_client.py` keeps only what a script of this repository calls plus the request
  machinery those calls need (`Request`, `send`/`call`, `GhApi`, `FakeGh`, the validators); every
  other request builder and helper and its tests are deleted. `test_gh_client.py` keeps the check
  that the fake and `gh api` build the same requests for every kept method, and the refusal
  checks for caller text.
- **Consequences:** No behavior change for `make catch-up` or `make renumber`. A later script
  that needs another GitHub request adds it back to `gh_client.py` with its fake-vs-`gh api`
  test, in the PR that first calls it. The python CI job runs fewer tests.
