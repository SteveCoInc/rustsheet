---
id: T-4279
title: "Speed up the full collation and collation keys on long texts (about 100-250 ns a character; T-4233 follow-up)"
milestone: M7
lane: perf
status: done
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
- [x] Each of the shapes above is at least 2x faster on a release build (before/after timings in the Outcome), and the
      tick weights are re-fitted to the new timings.
- [x] No ordering changes (collation tests, `make eval` no regressions); `make check` green; reviewers approve.

## Out of scope
- Stopping a node inside its comparisons (T-4278).

## Outcome
- `cmp_chars` (the full collation) keeps each text's elements from its first pass and runs the five later passes over
  them, instead of case-folding and decomposing every character again for each pass. A first-pass tie means both texts
  were read to the end, so nothing is read twice. `collation_key` makes the elements once for its five passes.
  The ASCII second pass is byte-wise (`ascii_skipped`), and `fold` and `is_weightless` have fast paths. Unit tests
  check each shortcut against the general path: `fold_fast_paths_are_identities` and
  `ascii_skipped_pass_matches_the_general_one`. No ordering change: the collation tests, the 5,387 engine unit tests
  and `make check-fast`'s eval gate pass (0 regressions).
- The case folds of SEARCH, the lookups and the criteria moved to `collation/case_fold.rs`, which keeps
  collation.rs under the 1,000-line limit.
- Timings, release build, 2026-11-05, ns per character of one text (the harness is in the PR description):

  | shape | before | after | speed-up |
  |---|---|---|---|
  | ASCII equal texts (second pass) | 81 | 3.2 | 25x |
  | ASCII case-only-different | 77 | 3.2 | 24x |
  | `é` equal, total order | 1,179 | 143-153 | 8x |
  | `é` equal, pairwise | 1,201 | 122-131 | 9x |
  | `中` equal | 1,300 | 140-151 | 9x |
  | `CollationKey`, ASCII | 203 | 48-50 | 4x |
  | `CollationKey`, `é` | 488 | 59-75 | 7x |
  | UNIQUE of 100 distinct 16,000-`é` texts | 1.97 s | 0.76 s | 2.6x |

  The ASCII first pass is unchanged at 1.3 to 2.0 ns.
- Tick weights re-fitted (`collation/compared.rs`, arrays.md): `CHAR_TICKS` 128 to 48. The second pass now uses
  `BYTE_TICKS` (1, was 32), and the separate `ELEMENT_TICKS` constant is gone. `KEY_TICKS` 128 to 64.
  - Key weight: 32 would fit the long keys, but a million 107-byte keys then run 13 s unrefused. So the weight is
    doubled for per-key overhead, and that shape is refused as before.
  - Time to `#VALUE!` for T-4233's MAP shapes: 2.8 s to 7.1 s. T-4233's first fit gave 4.9 s to 7.4 s.
  - `ROWS(UNIQUE(REPT("a",20)&SEQUENCE(1048576)))` computes in 5.9 s (was 9.8 s).
  - Every weight is lower than main's, so no formula is newly refused.
- Eval delta: none (known failures unchanged, 0 regressions).
