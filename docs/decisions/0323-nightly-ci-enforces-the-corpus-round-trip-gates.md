# 0323 — Nightly CI enforces the corpus round-trip gates

- **Status:** accepted (2026-11-06, T-4632)
- **Context:**
  - The corpus round-trip suites (`roundtrip-calipers`, `roundtrip-poi`, `roundtrip-corpus-bench`; T-4173, T-4629)
    hold M15's measured kept rates: 99.986%, 99.997% and 99.99999% on main.
  - CI fetches no corpus, so the PR gate checks only the fixtures round-trip suite. `make check` holds POI and
    corpus-bench only where a developer has fetched them, and Calipers is ungated for its run time.
  - The nightly job runs `make eval` as a report and fetches nothing. No job enforces any corpus round-trip baseline.
  - `.github/` is a protected path, so a new CI step needs this record.
- **Decision:**
  - After `make eval`, the nightly job fetches the calipers, poi and corpus-bench suites with
    `rustsheet-eval fetch`, so a failed download fails the job.
  - It then runs `rustsheet-eval roundtrip --check` on the three corpus round-trip suites by name, with a 300 s
    per-case timeout. Calipers' two 1.4M-formula workbooks take about 55 s of the 60 s default on a developer
    machine.
  - Both steps call the release binary that `make eval` built, through `scripts/eval_guard.sh` as the make targets
    do (decision 0041), rather than through a new Makefile target. The fetch retries three times, 30 s apart. The
    job has a 120-minute timeout.
  - A red nightly run is triaged by the steward every day, through the "Nightly eval run red" row of
    docs/process/agent-workflow.md and the steward skill. No issue is opened automatically, unlike fuzz
    (decision 0302).
  - A regression or an out-of-date baseline fails the nightly run on main.
- **Consequences:**
  - A change that loses corpus content, or slows a case past its timeout, turns the nightly job red within a day. It
    does not block the PR that caused it; the steward's daily check of the run picks the failure up.
  - The nightly job takes a few minutes longer: the fetches, and the round trips (3 min 15 s on a developer machine;
    the runner's time is measured by the first run).
  - The run's `roundtrip.{json,md}` artifact is the corpus check's, not `make eval`'s, which covers the fixtures.
  - The corpus fetch hosts (github.com for Calipers and POI, raw.githubusercontent.com for SpreadsheetBench) are
    already the fetch specs' pinned sources. No new host or secret is added.
  - Moving the corpus gates into the PR gate stays open. It needs a cached fetch in CI and a time budget.
