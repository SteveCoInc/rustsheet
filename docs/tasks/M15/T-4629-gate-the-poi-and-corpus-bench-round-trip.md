---
id: T-4629
title: "Gate the POI and corpus-bench round-trip suites where their caches are fetched, as their cell suites are gated"
milestone: M15
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4173]
---

## Goal
The POI and corpus-bench round-trip suites are gated wherever their corpora are fetched, as their cell suites are, so
`make check` holds their committed kept rates.

## Context
- T-4173 added the corpus round-trip suites with `gate = false` and a `root` pointing into another suite's cache.
- A gated suite with a missing root fails `--check`. CI fetches no corpus, so these suites could not be gated that
  way. Their cell suites (`poi`, `corpus-bench`) are gated and skipped while not fetched.
- The goal for M15 asks for a gated 99% figure.

## Acceptance criteria
- [x] A round-trip manifest can name its cell suite (`cases_of`). It then takes that suite's `[fetch]` and cache
      directory, and is fetched, skipped and gated as that suite is (unit test).
- [x] `roundtrip-poi` and `roundtrip-corpus-bench` are `gate = true`. `roundtrip --check` runs them when fetched and
      skips them otherwise, touching no baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- Gating Calipers, whose cell suite is ungated for its run time.
- Fetching corpora in CI.

## Outcome
- `SuiteManifest::cases_of` is added. `Suite::load_roundtrip` copies the named suite's `[fetch]` and cache directory,
  and refuses a name that is no suite, a suite that is not fetched, or a manifest with both.
- The three corpus round-trip manifests use `cases_of` instead of a `root` into the cache. POI and corpus-bench are
  gated.
- `roundtrip --check` with the caches fetched: fixtures, poi and corpus-bench pass in about 20 s. Without them, the
  three corpus suites are skipped with "run `rustsheet-eval fetch --suite poi`" (the cell suite's name), and the
  fixtures alone are checked.
