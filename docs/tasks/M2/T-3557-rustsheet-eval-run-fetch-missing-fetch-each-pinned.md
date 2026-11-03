---
id: T-3557
title: "rustsheet-eval run --fetch-missing: fetch each pinned external suite that is missing or stale before scoring, instead of skipping it"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-eval run --fetch-missing` (and every other subcommand that selects suites through `Selection`) fetches each
selected external suite whose `fetch_state()` is `NotFetched` before it scores anything. A full run on a fresh
checkout then scores poi, calipers and corpus-bench in one pass, with no second `make eval`.

## Context
- Today `select` in crates/rustsheet-eval/src/main.rs drops every `FetchState::NotFetched` suite with
  "skipping <name>: not fetched (run `rustsheet-eval fetch --suite <name>`)". On a fresh container the steward
  therefore runs `make eval`, then `rustsheet-eval fetch` for the three suites, then `make eval` again
  (docs/status.md, 2026-10-31 18:43 UTC snapshot: "the first `make eval` skipped them; I ran `rustsheet-eval
  fetch` for the three and ran `make eval` again"). A planner run on b39e1eae hit the same skip, and
  `make phase1-exit` reported `run not met` for it.
- The fetch code already exists: `fetch_suite` in crates/rustsheet-eval/src/fetch.rs (pinned `rev` or `sha256`,
  staged and renamed into `cache_dir`, zip-size cap). `Cmd::Fetch` in main.rs is the only caller. Reuse
  `fetch_suite`; do not copy its loop.
- `FetchState::NotFetched` covers both a missing cache and a stale pin (the marker differs from `spec.pin()`,
  suite.rs `fetch_state`), so the flag refreshes a stale cache too.
- This is a CLI flag only. Whether `make eval` passes it, and whether CI fetches, is T-3556's spec
  (Makefile and `.github/` are protected paths); this task does not touch them.

## Acceptance criteria
- [x] `Selection` gains `--fetch-missing` (doc comment on the flag). With it, `select` calls `fetch_suite` for each
      selected suite that is `NotFetched`, prints `fetched <name> (<n> files)`, and keeps the suite. Without it,
      behavior and output are unchanged.
- [x] A failed fetch (bad URL, hash mismatch, no network) does not abort the run: it prints
      `skipping <name>: fetch failed: <error>` and skips the suite as today. The exit code is unchanged.
- [x] Unit tests in the eval crate, built like fetch.rs's `archive_fetch_verifies_pin_filters_and_marks_fetched`
      (local archive, no network): a missing suite is fetched and selected; a stale marker is refetched; a hash
      mismatch is skipped with the message above and the other suites still run.
- [x] `rustsheet-eval run --help` lists the flag. docs/architecture/eval-harness.md, where it
      documents `rustsheet-eval fetch`, describes the flag.
- [x] `make check` green; eval 0 regressions, 0 fixed; reviewers approve.

## Out of scope
- Makefile, CI and steward-skill changes that use the flag (T-3556's rollout).
- Fetching suites the selection does not name, or retrying failed downloads.

## Outcome
`Selection` gains `--fetch-missing` (run, check, update-baseline, show-free paths). `select` fetches each selected
`NotFetched` suite (missing or stale marker) through `fetch::fetch_missing`, which wraps `fetch_suite` and returns the
`skipping <name>: fetch failed: <error>` line on failure; the suite is then skipped and the exit code is unchanged.
Unselected suites and runs without the flag behave as before. Unit test
`fetch_missing_fetches_missing_and_stale_and_skips_failures` covers missing, stale and hash-mismatch with local
archives. Documented in docs/architecture/eval-harness.md. Eval delta: 0 fixed, 0 regressions.
