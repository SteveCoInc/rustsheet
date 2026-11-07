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
  - **Cache and fetch:**
    - CI's `eval` job restores `eval/.cache/suites/poi` and `eval/.cache/suites/corpus-bench` with
      `actions/cache`. The key is `fetched-suites-v1-` plus the hash of the two suites' `suite.toml` (their pins),
      with no `restore-keys`.
    - On a miss, the job runs `rustsheet-eval fetch --suite poi --suite corpus-bench` through
      `scripts/eval_guard.sh`, three tries 30 s apart. A fetch that fails three times fails the job.
  - **Gating:** the job runs `make eval-gates` with `EVAL_GATE_FLAGS=--require-fetched` (T-4725). A selected gate
    suite that is still not fetched then fails the run, never skips.
  - **Fetching:** the job sets `EVAL_FETCH` empty, so no make target fetches in CI. Locally `EVAL_FETCH` defaults
    to `1`:
    - `eval-check`, `eval-gates` and `eval-baseline` first run `fetch --missing --gates`, ignoring its failure;
    - `eval` passes `--fetch-missing` to `run` and `roundtrip` (SPEC-0035 D3).
  - **calipers:** stays ungated (`gate = false`, SPEC-0035 D4).
- **Consequences:**
  - A regression in poi or corpus-bench cells or round trips fails the PR that causes it. A PR that fixes one of
    their cells must record it with `make eval-baseline`, which now fetches them first.
  - The `eval` job has a SPEC-0035 D1 budget of at most 90 s extra, under 7 min in all. T-4726's Outcome records
    the measured times for a cache miss and a hit. Over budget, the change goes back to SPEC-0035.
  - `make check` (through `eval-gates`) fetches poi and corpus-bench once on a fresh checkout, about 16 s. Offline,
    the pre-fetch's failure is ignored, they skip as before, and CI judges them.
  - `make eval` locally also fetches and scores calipers (about 8 min, SPEC-0035 D3). The CI report step does not.
  - Decision 0067 stands for its three MODE cells. Only its remark that `make check` and CI do not run poi is
    superseded.
