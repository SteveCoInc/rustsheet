---
id: T-4725
title: "rustsheet-eval check/roundtrip --require-fetched: a selected gate suite that is not fetched fails the run instead of being skipped (SPEC-0035)"
milestone: M2
lane: harness
status: todo
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
- [ ] `check --require-fetched` and `roundtrip --check --require-fetched` exit non-zero, naming each selected
      gate suite that is not fetched (or whose cache is stale), and score nothing else differently.
- [ ] An ungated suite (`gate = false`, e.g. calipers) that is not fetched is still skipped under the flag.
- [ ] `rustsheet-eval fetch --missing --gates` (SPEC-0035 D3) fetches each gate suite whose cache is missing or
      stale (`fetch_state()`), skips a current cache and every ungated suite, and exits non-zero naming each suite
      whose fetch failed. `--missing` and `--gates` also work alone (`--gates` alone fetches every gate suite).
- [ ] Unit tests cover: unfetched gate suite fails; fetched gate suite passes; unfetched ungated suite skipped;
      the flag conflicts with `--fetch-missing`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Wiring the flags into the Makefile or CI (T-4726).
