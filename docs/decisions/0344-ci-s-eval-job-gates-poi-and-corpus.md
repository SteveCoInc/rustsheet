# 0344 — CI's eval job gates poi and corpus-bench from a cached pinned fetch; local eval targets fetch missing gate suites

- **Status:** accepted (2026-11-07, T-4726)
- **Context:**
  - SPEC-0035 (T-3556) decides that the fetched gate suites, poi and corpus-bench, gate every PR and merge group.
    That covers their cells and their `roundtrip-*` suites. CI fetched no corpus, so its required `eval` job
    printed `skipping poi|corpus-bench: not fetched` and passed (SPEC-0035 "Problem").
  - The cost of that gap: T-4176 (#3800) lost 21% of `roundtrip-poi/54436.xlsx` with CI green. The loss was
    later recorded as a known failure by a re-record (T-4825, T-4961). T-3120's calipers regression got through
    the same way.
  - `.github/workflows/ci.yml` and `Makefile` are protected paths. This also amends decision 0067's "the poi suite
    is fetched, not vendored, so `make check` and CI do not run it".
- **Decision:**
  - **Two parallel jobs (amends SPEC-0035 D1):**
    - The gate runs as `eval (vendored suites)`, the job as it was, which fetches nothing, and
      `eval (fetched gate suites)`.
    - A summary job keeps the required check name, `eval (Excel-compatibility gate)`. It passes only when both
      jobs passed, or both were skipped on purpose (a docs-only PR, decision 0049; a push to main, decision
      0249), and fails on any other result.
    - Why: in one job the corpora took the `eval` job to 8 min 19 s on a cache hit and 8 min 30 s on a miss, past
      D1's 7 min. The round trips cost 82 s since R1 (T-4514), and the cells 28 s (PR #3999's runs; main's job
      that day took 5 min 36 s).
  - **Cache and fetch:**
    - The fetched job restores `eval/.cache/suites/poi` and `eval/.cache/suites/corpus-bench` with
      `actions/cache`. The key is `fetched-suites-v1-` plus the hash of the two suites' `suite.toml` (their pins),
      with no `restore-keys`.
    - On a miss, the job runs `rustsheet-eval fetch --suite poi --suite corpus-bench` through
      `scripts/eval_guard.sh`, three tries 30 s apart. A fetch that fails three times fails the job.
  - **Gating:** the fetched job runs `make eval-fetched-gates` with `EVAL_GATE_FLAGS=--require-fetched` (T-4725):
    - `check` over poi and corpus-bench;
    - `roundtrip --check` over `roundtrip-poi` and `roundtrip-corpus-bench`.

    A gate suite still not fetched fails the run, never skips. The vendored job runs `make eval-gates` as before,
    where the fetched suites skip.
  - **Fetching:** the job sets `EVAL_FETCH` empty, so no make target fetches in CI. Locally `EVAL_FETCH` defaults
    to `1`:
    - `eval-check`, `eval-gates` and `eval-baseline` first run `fetch --missing --gates`, ignoring its failure;
    - `eval` passes `--fetch-missing` to `run` and `roundtrip` (SPEC-0035 D3).
  - **calipers:** stays ungated (`gate = false`, SPEC-0035 D4).
- **Consequences:**
  - A regression in poi or corpus-bench cells or round trips fails the PR that causes it. A PR that fixes one of
    their cells must record it with `make eval-baseline`, which now fetches them first.
  - Each PR uses one more runner, about 6 min of it, as decision 0122's shards did. PR wall time stays about what
    the one `eval` job took. Each of the two jobs must stay under 7 min (SPEC-0035 D1, amended); T-4726's Outcome
    records their times.
  - The fetched job names its suites (`make eval-fetched-gates`, the cache paths). Gating a new fetched suite means
    adding it to both places, and a suite renamed without that is not checked there. The cache key's `suite.toml`
    paths change with it, so review catches the edit.
  - `make check` (through `eval-gates`) fetches poi and corpus-bench once on a fresh checkout, about 16 s. Offline,
    the pre-fetch's failure is ignored, they skip as before, and CI judges them.
  - `make eval` locally also fetches and scores calipers (about 8 min, SPEC-0035 D3). The CI report step (in the
    vendored job) fetches nothing, so its report skips the fetched suites, as before.
  - Decision 0067 stands for its three MODE cells. Only its remark that `make check` and CI do not run poi is
    superseded.
