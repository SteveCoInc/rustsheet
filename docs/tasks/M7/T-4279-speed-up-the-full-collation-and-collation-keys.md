---
id: T-4279
title: "Speed up the full collation and collation keys on long texts (about 100-250 ns a character; T-4233 follow-up)"
milestone: M7
lane: perf
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The full collation (non-ASCII text) and collation keys cost closer to the ASCII first pass a character, so text
comparisons over long texts do more work per second of the formula work budget.

## Context
Measured for T-4233 (release build, 2026-11-05, ns per character read): the ASCII first pass about 1.6, the ASCII second
pass (`elements`, equal or case-only-different texts) about 14 per counted item, the full collation of `é` or `中` text
95 to 140, a `CollationKey` (`collation_key`, UNIQUE, the exact-match index, the *IF(S) cache) about 224 a byte for
ASCII, and a UNIQUE of 100 distinct 16,000-`é` texts 2.9 s. The work budget's tick weights in
`functions/util/collation/compared.rs` follow these timings; any speed-up lets their weights come down (re-time per
SPEC-0024 section 4). Profile `cmp_chars`, `elements` and `collation_key` on these shapes and remove the per-character
overhead (allocation, repeated decomposition) without changing any ordering (the collation unit tests and eval cases
are the guard).

## Acceptance criteria
- [ ] Each of the shapes above is at least 2x faster on a release build (before/after timings in the Outcome), and the
      tick weights are re-fitted to the new timings.
- [ ] No ordering changes (collation tests, `make eval` no regressions); `make check` green; reviewers approve.

## Out of scope
- Stopping a node inside its comparisons (T-4278).
