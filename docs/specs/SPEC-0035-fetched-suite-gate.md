# SPEC-0035 — Fetched-suite gate

- **Status:** accepted (2026-11-06, T-3556)
- **Milestone / lane:** M2 / harness
- **Tasks:** T-3556 (this spec); rollout T-4725, T-4726, T-4727, T-4728

## Problem
Three cell suites are fetched, not vendored: poi, calipers and corpus-bench (`[fetch]` in their suite.toml). Neither
PR CI nor the merge queue scores them:
- The `eval` job of `.github/workflows/ci.yml` runs `make eval-gates` in a fresh checkout. `select` in
  crates/rustsheet-eval/src/main.rs skips every suite whose `fetch_state()` is `NotFetched`. The job log of CI run
  42312941695 (2026-11-06) prints `skipping calipers|corpus-bench|poi: not fetched` and the same for the three
  `roundtrip-*` corpus suites.
- poi and corpus-bench say `gate = true`, but that gate holds only on a machine that has fetched them. calipers says
  `gate = false`.
- A regression got through this way: T-3120 (#2389) moved calipers `roundtrip/formulas_financial` Sheet1!E132 (IRR)
  2 ulps off Excel with CI green. It surfaced the next day, when `update-baseline --suite calipers` refused it
  (T-3246).
- Phase 1 reads these suites (Gate B, decision 0188 R1(b); M8's poi subset). `make eval` skips them on a clean
  checkout, so `make phase1-exit` prints `run not met ... skipped suites: calipers, corpus-bench, poi`.
- Since decision 0323 (T-4632), the nightly job fetches all three and gates their **round-trip** suites. It still
  scores none of their **cells**. Its `make eval` step runs before the fetch, so it skips them too.

## Measurements
All on commit 826c44290 (origin/main, 2026-11-06), release binary. "Runner" means GitHub's `ubuntu-24.04` runner.
"Local" means a 4-core, 15 GB Linux container (this task's session).
- **Runner/local speed.** Over the same vendored `check --evidence-floor`, the runner took 76 s (CI run 42312941695,
  log timestamps 23:39:37.97 to 23:40:53.90) and local took 61 s. Runner ≈ 1.25 × local, and the runner estimates
  below use that factor.
- **Fetch, runner.** All three suites took 16 s together: the fetch step of nightly run 42242940440, 15:31:10 to
  15:31:26. `fetch` always downloads again; it does not skip a current cache.

| suite | files | cache | fetch, local | `check`, local | `check`, runner (est.) | `roundtrip --check`, local |
|---|---|---|---|---|---|---|
| poi | 366 | 22 MB | 9.5 s | 1.8 s | ~2.3 s | 11.8 s |
| corpus-bench | 912 | 33 MB | 6.3 s | 4.3 s | ~5.3 s | 12.6 s |
| calipers | 541 | 103 MB | 7.5 s | 463 s | ~580 s (~9.7 min) | (nightly, 0323) |

- **The `eval` job's time today.** 5 min 17 s on the runner (job 128218753666), of which `make eval-gates` is
  3 min 47 s.
- **poi and corpus-bench** therefore add about 16 s of fetch on a cache miss, 0 s on a hit, plus about 8 s of cell
  check and about 30 s of round-trip check (the latter scaled from local). That is under 1 min, or about 15% of the
  job.
- **calipers** adds about 9.7 min, nearly three times the job. Its `check` also fails today with nothing regressed:
  `known failures 598 -> 593 (fixed 5, REGRESSIONS 0)`, then `baseline is out of date`. Engine PRs fix calipers cells
  without recording them, because nothing requires them to.
- Both cell baselines are current on main: poi `111 -> 111`, corpus-bench `3 -> 3`, 0 fixed. Gating them adds no
  catch-up work on day one.

## Goals / non-goals
Goals:
- A regression in poi or corpus-bench, cells or round trip, fails the PR and the merge group, as a vendored suite's
  regression does.
- A regression in calipers cells turns a scheduled job red within a day, without waiting for the steward's next
  `update-baseline`.
- Neither a failed download nor a missing cache passes a gate silently.
- `make eval` measures every suite Phase 1 reads.

Non-goals:
- Fixing any cell of these suites.
- Vendoring them (decision 0019: licenses and size).
- Counting fetched suites in the evidence floor or the visible-parity lock (D5).

## Design

### D1. Which suites gate a PR, and where
- **poi and corpus-bench gate in the required `eval` job**, which the merge group runs too. No separate or
  path-filtered job is added:
  - their cost (under 1 min) is less than a second job's setup and release build (about 2.5 min, the job's
    `cargo build` step);
  - the `changes` job already skips `eval` on docs-only diffs (decision 0249);
  - a path filter would miss regressions from shared code, which is how T-3120's got through.
- **calipers does not gate a PR.** It stays `gate = false` (D4). The nightly job checks its cells (D4).
- **Time budget.** The `eval` job may grow by at most 90 s on the runner, about 5 min 17 s to under 7 min. T-4726
  states the measured time of its first CI run. If the job exceeds 7 min, T-4726 does not land as written; it goes
  back to this spec.
- **Amended (decision 0344, T-4726): two parallel jobs.**
  - **Why:** the one-job design measured over budget in T-4726's CI runs (PR #3999).
    - The `eval` job took 8 min 30 s on a cache miss and 8 min 19 s on a hit, against 5 min 36 s on main the
      same day.
    - The added work is the corpora's round trips, which R1 (T-4514) doubled after this spec was measured (+82
      s), and their cells (+28 s).
    - That made `eval` the PR's slowest job by about 3 min; the rust shards took 5 min 31 s.
  - **The jobs:** CI now runs the gate as two parallel jobs.
    - `eval (vendored suites)` is the job as it was.
    - `eval (fetched gate suites)` does D2's cache and fetch, then `make eval-fetched-gates`.
    - `eval (Excel-compatibility gate)` is a summary job, as decision 0122's `rust (fmt, clippy, test)` is. It
      keeps the required check name, passes only when both jobs passed or both were skipped on purpose, and
      fails otherwise.
  - **The release build:** the second job's release build is the cost this section weighed against one job. It
    costs runner time, not PR wall time, because each job takes about as long as the one job used to.
  - **The budget now:** each of the two jobs stays under 7 min. T-4726's Outcome records their times.

### D2. Cache key and fetch failure
- **Cache.** `actions/cache` restores `eval/.cache/suites/poi` and `eval/.cache/suites/corpus-bench`. The key is
  `fetched-suites-v1-` plus `hashFiles('eval/suites/poi/suite.toml', 'eval/suites/corpus-bench/suite.toml')`.
  - The pins (`rev`, `sha256`, `subdir`, `include`) live in those files, so a new pin is a new key.
  - Hashing the whole file is deliberately coarse. An edit that leaves the pin alone costs one 16 s refetch.
  - No `restore-keys`: a partial match would restore a cache from another pin.
- **Fetch on a miss.** When `cache-hit` is not `true`, a step runs `rustsheet-eval fetch --suite poi --suite
  corpus-bench` through `scripts/eval_guard.sh`. It retries 3 times, 30 s apart, as the nightly fetch does (decision
  0323).
- **Failure fails the job.** A fetch that fails three times fails the `eval` job. It never skips:
  - a skip in a required job reads as green, which is exactly how T-3120's regression got through;
  - the hosts are github.com and raw.githubusercontent.com, which the job already depends on for checkout;
  - a cache hit needs no network at all.
- **Belt and braces.** The job's `eval-gates` passes `--require-fetched` (T-4725). A gate suite that is selected and
  not fetched then fails the run instead of printing `skipping`. This catches a broken cache path or a renamed suite,
  which the fetch step cannot see.
- `--fetch-missing` (T-3557) is **not** used in CI. It skips a suite whose fetch fails, which D2 forbids.
- **Round trips.** `roundtrip-poi` and `roundtrip-corpus-bench` read the same two caches (`cases_of` in their
  eval/roundtrip suite.toml) and say `gate = true`, so the restored cache gates them too. `roundtrip-calipers` says
  `gate = false`, so `--require-fetched` does not trip on its missing cache.

### D3. `make eval` and the local targets
Two Makefile variables carry the per-environment behavior. Every target that runs the harness states its behavior
in both environments below.
- **`EVAL_FETCH`**, default `1` locally. ci.yml sets it empty for the whole `eval` job, so no make target fetches
  in CI. CI's own fetch step (D2) is the only fetch there.
- **`EVAL_GATE_FLAGS`**, default empty locally. ci.yml sets it to `--require-fetched` (T-4725).
- **Gate-suite pre-fetch.** With `EVAL_FETCH` set, `eval-check`, `eval-gates` and `eval-baseline` first run
  `rustsheet-eval fetch --missing --gates` (T-4725).
  - It fetches only the gate suites (poi and corpus-bench today; never calipers) whose cache is missing or stale.
  - The Makefile ignores its failure (a `-` prefix). Offline, the gate suites then skip with their `skipping` line
    as today, and CI judges them (D2).
  - `run --fetch-missing` and `update-baseline --fetch-missing` would also fetch calipers (they select every suite;
    `select_from` in main.rs), so the baseline targets do not use them.

| target | locally (`EVAL_FETCH=1`) | in CI (`EVAL_FETCH=`, `EVAL_GATE_FLAGS=--require-fetched`) |
|---|---|---|
| `eval-check` | pre-fetch gate suites; `check` | not run by CI |
| `eval-gates` | pre-fetch gate suites; `check --evidence-floor`, `accuracy`, `roundtrip --check` | no fetch; `check --evidence-floor --require-fetched` and `roundtrip --check --require-fetched`; `accuracy` unchanged |
| `eval-baseline` | pre-fetch gate suites; `update-baseline`, `roundtrip --update-baseline` | not run by CI |
| `eval` (report) | `run --fetch-missing` and `roundtrip --fetch-missing`: every suite, calipers included | no fetch; scores poi and corpus-bench from the job's cache and skips calipers, as today |

- **`accuracy`** gets no flag. It reads the reference tables in eval/accuracy only (crates/rustsheet-eval/src/
  accuracy.rs), never a fetched suite.
- **Cost of the local report.** `make eval` grows by about 25 s of first fetch and about 8 min of calipers (463 s
  locally), plus its round trip (about 3 min, decision 0323). That is the price of the goal "`make eval` measures
  what Phase 1 reads": Gate B counts calipers. `make check` and `make check-fast` never run `make eval`, so the
  pre-push gate does not pay it.
- **Cost of CI's report step.** "Full report (informational)" (ci.yml) runs `make eval` on a failure and on each
  push to main. It gains the poi and corpus-bench cells and round trips, about 40 s on the runner, and no calipers.
  It is not part of D1's PR budget, because it runs only on a failed gate or on main.
- **Baselines.** A PR that fixes a poi or corpus-bench cell records it with `make eval-baseline`, which the
  pre-fetch makes possible and D1 requires once they gate. A developer who never fetched calipers leaves
  eval/baseline/calipers.txt alone, as today; the steward ratchets it (D4).
- **`make eval` on the nightly job** runs after the fetch step instead of before, so the nightly report scores every
  suite (T-4728).
- **`make phase1-exit` and the progress dashboard keep their rule.** A run with any `run.skipped_suites` is
  `run not met` (scripts/phase1_exit.py), and the dashboard shows it as incomplete
  (tools/src/rustsheet_tools/progress_report/history.py, `complete: False`). Nothing changes there: with D3 a
  skipped suite means a failed fetch, and that should stay visible.

### D4. calipers
- **It stays `gate = false`.** At about 9.7 min on the runner it would nearly triple the required job, and its
  baseline is already out of date on main (5 fixed cells unrecorded). If it gated, every PR that moved a calipers
  cell would have to run it locally (about 8 min) to ratchet the baseline.
- **The nightly job checks its cells instead** (T-4728). After the fetch, it runs `rustsheet-eval check --suite
  calipers --timeout 300 --regressions-only` (T-4727).
  - The run fails on a regression and prints, without failing, a baseline that is only out of date.
  - The 300 s timeout is decision 0323's, for the two 1.4M-formula workbooks.
  - A red nightly run goes to the steward's existing triage row ("Nightly eval run red", decision 0323). It turns a
    T-3120-style regression around within a day, with the offending range of merges known.
- **`--regressions-only` is for calipers only.** It is never used on a gate suite, because a stale baseline there
  must fail (directive 2). Without it, every PR that fixed a calipers cell would turn the nightly red until the next
  ratchet, and a run that is red most days hides the regression it exists to show.
- **Baseline and overrides.** eval/baseline/calipers.txt and `cached-overrides.tsv` are handled as today. The steward
  ratchets the baseline with `update-baseline --suite calipers` (T-3230), and an override changes only through its
  decision (0090, 0133). The nightly check reads both and writes neither.
- **When calipers would gate a PR.** Only once the eval job's calipers check fits the D1 budget. That needs about a
  6× speed-up, beyond this spec. A later spec may revisit it if the merge group gets a longer budget.

### D5. Evidence floor and visible-parity lock
Neither starts counting fetched suites. Decisions 0062 and 0088 stand:
- Both locks are ratchets that every checkout must reproduce, including one with no network. A count that moved
  with a failed fetch would make the floor flap.
- poi and corpus-bench cells gain a hard gate through their own baselines (D1), which holds them as tightly as
  the lock would.
- Calipers is Phase 1's Gate B input, read from the full run (D3), not a per-PR lock.

## Alternatives considered
- **Gate all three in the required job.** Rejected for calipers: about 9.7 min, and an out-of-date baseline would
  fail every unrelated PR on day one.
- **A separate, path-filtered `fetched-eval` job.** Rejected: a second release build costs more than the suites, and
  a path filter misses shared-code regressions.
- **Run poi and corpus-bench only in the merge group.** Rejected: a PR could then be enqueued, fail in the queue and
  be dropped, and the author would learn of it later. That is decision 0253's `queue-dropped` cost, paid for 24 s
  of work.
- **Skip with a warning on a failed fetch.** Rejected (D2): a green required job that skipped a gate is what let
  T-3120 through.
- **`--fetch-missing` in CI.** Rejected (D2): it skips on failure.

## Test plan
- T-4725: unit tests in crates/rustsheet-eval. A selected unfetched gate suite fails `check --require-fetched` and
  `roundtrip --check --require-fetched` with a message naming it. An ungated or fetched suite is unaffected.
  `fetch --missing --gates` fetches a missing or stale gate suite, leaves a current cache and every ungated suite
  alone, and exits non-zero naming a suite whose fetch failed.
- T-4727: unit tests. `--regressions-only` exits 0 on an out-of-date baseline with no regression, prints the
  out-of-date line, and exits 1 on any regression.
- T-4726: locally, `make eval-baseline` on a clean checkout fetches poi and corpus-bench and not calipers. The PR's own CI run shows `[poi]` and `[corpus-bench]` lines and no `skipping poi|corpus-bench`. The PR
  records the job time against D1's budget. A second run shows the cache hit.
- T-4728: a `workflow_dispatch` run of the nightly on the branch shows the calipers check's summary line.
- Hand test (each task's Outcome): delete one poi cell from eval/baseline/poi.txt in a scratch branch and confirm the
  CI eval job fails.

## Rollout
Protected paths need a decision record (`make new-decision`, CLAUDE.md directive 2).

| # | task | size / tier | protected paths | decision record |
|---|---|---|---|---|
| 1 | T-4725 `check`/`roundtrip --require-fetched`; `fetch --missing --gates` | S / sonnet | none (crates/rustsheet-eval) | none |
| 2 | T-4727 `check --regressions-only` | S / sonnet | none (crates/rustsheet-eval) | none |
| 3 | T-4726 CI gates poi and corpus-bench; local targets fetch missing suites | M / opus | `.github/workflows/ci.yml`, `Makefile`, `docs/process/quality-gates.md` | yes: amends 0067's "`make check` and CI do not run" poi |
| 4 | T-4728 nightly checks calipers cells; `make eval` after the fetch | S / opus | `.github/workflows/nightly.yml`, `docs/process/agent-workflow.md` (the nightly triage row) | yes: amends 0323 |

- T-4726 depends on T-4725. T-4728 depends on T-4727. Tasks 1 and 2 are independent.
- **The stale `nightly.yml` comment** ("downloaded here once their fetchers exist (T-0006)"), which T-3556's context
  cites, is gone on main: T-4632 replaced the job's steps under decision 0323 (`rg T-0006 .github` finds nothing). T-4728
  re-checks the file and removes any leftover when it edits the nightly job.
- Until T-4726 lands, the steward's hand fetch and second `make eval` stay as they are.
