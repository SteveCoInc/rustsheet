# 0346 — The nightly eval fetches, runs and checks the pivot-corpus suites

- **Status:** accepted (2026-11-07, T-4906).
- **Context:**
  - The pivot-corpus suites `pivot-corpus-lo` and `pivot-corpus-closedxml` (T-4131) are fetched (`[fetch]`) and
    `gate = false`, so no PR gate runs them and `make eval` skips them on a fresh runner. Until now only a local run
    measured them.
  - T-4906 makes them score the re-render of every fresh pivot (plan check (b)) as cells with cell baselines, and
    report the refresh of the `refreshOnLoad` ones (check (c)). The M16 exit row's corpus render measure (T-4127,
    wired by T-4971) needs those counts from a run anyone can read, not from one agent's machine.
  - The nightly already fetches calipers, poi and corpus-bench in its own retried step for their round-trip gates
    (decision 0323). The two corpora are small (83 workbooks, about 10 MB) and run in seconds.
- **Decision:**
  - The nightly's fetch step also fetches `pivot-corpus-lo` and `pivot-corpus-closedxml`, in the same three-attempt
    loop, so a failed download fails the job instead of skipping a suite.
  - A step after it runs both suites under `eval_guard.sh` into `eval/runs/latest/pivot-corpus/`, which the
    artifact keeps (report.md "Pivot freshness", report.json `fresh_pivot_render`).
  - A last step checks their cell baselines (`rustsheet-eval check --suite ...`) whatever the round-trip step gave
    (`if: !cancelled()`), so a calipers round-trip timeout cannot hide a pivot-corpus regression.
- **Consequences:**
  - This adds a nightly gate and weakens none. The PR gate is unchanged: the suites stay `gate = false` until they
    are shown fast and stable in CI, and their baselines are still ratcheted with
    `rustsheet-eval update-baseline --suite ...` (or `make eval-baseline` with the corpora fetched).
  - A nightly failure of the new check is a regression of an Excel-saved pivot's render or of a formula cell of
    the corpus, and is handled as any other nightly failure (the steward skill).
