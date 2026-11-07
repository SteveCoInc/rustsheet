---
id: T-4725
title: "rustsheet-eval check/roundtrip --require-fetched: a selected gate suite that is not fetched fails the run instead of being skipped (SPEC-0035)"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-3556]
---

## Goal
A gate suite that a run selects but has not fetched fails `rustsheet-eval check` and `roundtrip --check` when
`--require-fetched` is given, instead of being skipped with a `skipping <suite>: not fetched` line.

## Context
- docs/specs/SPEC-0035-fetched-suite-gate.md D2 ("Belt and braces"): CI's eval job fetches poi and corpus-bench itself (T-4726) and must never pass
  with one of them silently skipped (a broken cache path, a renamed suite).
- Today `select` in crates/rustsheet-eval/src/main.rs skips every suite whose `fetch_state()` is `NotFetched`;
  `roundtrip.rs` does the same for `roundtrip-*` suites.
- `--fetch-missing` (T-3557) is the local counterpart; the two flags conflict (clap `conflicts_with`).

## Acceptance criteria
- [x] `check --require-fetched` and `roundtrip --check --require-fetched` exit non-zero, naming each selected
      gate suite that is not fetched (or whose cache is stale), and score nothing else differently.
- [x] An ungated suite (`gate = false`, e.g. calipers) that is not fetched is still skipped under the flag.
- [x] `rustsheet-eval fetch --missing --gates` (SPEC-0035 D3) fetches each gate suite whose cache is missing or
      stale (`fetch_state()`), skips a current cache and every ungated suite, and exits non-zero naming each suite
      whose fetch failed. `--missing` and `--gates` also work alone (`--gates` alone fetches every gate suite).
- [x] Unit tests cover: unfetched gate suite fails; fetched gate suite passes; unfetched ungated suite skipped;
      the flag conflicts with `--fetch-missing`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Wiring the flags into the Makefile or CI (T-4726).

## Outcome
- **New module:** `crates/rustsheet-eval/src/select.rs` now holds `Selection` and `select_from`, moved out of main.rs.
  main.rs was at 997 of the 1,000 lines the ratchet allows, and the library makes them unit-testable.
  `select_from` returns the `fetched`/`skipped` lines, because only the binary prints.
- **`--require-fetched`:** `check`, `roundtrip`, and every other subcommand with a `Selection` accept the flag.
  - A selected gate suite whose cache is missing or stale fails the run with
    `--require-fetched: gate suites not fetched or stale: poi (`fetch --suite poi`)`, naming every such suite.
  - A round-trip suite over a cell suite's cache (`cases_of`) is named with that cell suite, so
    `roundtrip-poi` points at `fetch --suite poi`.
  - Ungated and unselected suites are skipped as before. The flag conflicts with `--fetch-missing` (clap).
- **`fetch --missing` / `--gates`:** `select::to_fetch` picks the suites.
  - `--missing` leaves a current cache alone and prints `<suite> is current`.
  - `--gates` takes only `gate = true` suites, so never calipers.
  - Every chosen suite is now tried, and the run exits non-zero naming each one whose fetch failed. A plain `fetch`
    used to stop at the first failure.
- **Tests:** six unit tests in `select.rs`:
  - an unfetched gate suite fails, with every name in the message;
  - a fetched gate suite passes;
  - an unfetched ungated suite is skipped, by default and when named;
  - a stale gate cache fails;
  - an unselected unfetched gate suite does not fail;
  - the flag conflicts with `--fetch-missing`;
  - `to_fetch` covers `--missing --gates`, each flag alone, neither, and `--suite`.
- **By hand, release binary, poi and corpus-bench fetched:**
  - `fetch --missing --gates` prints `corpus-bench is current` and `poi is current`, and exits 0.
  - With `eval/.cache/suites/poi` moved aside, `check --require-fetched --suite poi` and
    `roundtrip --check --require-fetched` exit 1, naming `poi` and `roundtrip-poi`.
  - Without the flag, `roundtrip --check --suite roundtrip-fixtures` runs as before.

