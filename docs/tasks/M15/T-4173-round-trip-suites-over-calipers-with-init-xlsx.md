---
id: T-4173
title: "Round-trip suites over calipers (with init.xlsx), poi, corpus-bench and fixtures, with fidelity baselines"
milestone: M15
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4172, T-4509, T-4513, T-4514]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
Round-trip suites over Calipers (all goldens; `roundtrip/` scored against `init.xlsx` → Excel golden), poi,
corpus-bench and the fixtures, each with a fidelity baseline, so every later M15 change is measured.

## Context
Plan: docs/research/xlsx-round-trip-2026-11-04.md §6.2. The calipers fetch includes only `**/golden.xlsx` today (eval/suites/calipers/suite.toml), so widen
it to `init.xlsx` for the `roundtrip/` cases. Corpus-bench and calipers are not in `make check` for time reasons, so
follow their gating pattern.

## Acceptance criteria
- [x] The four suites run and baselines are committed.
- [x] The eval report shows the fidelity table by feature and corpus. Its first numbers are copied into the M15
      section of docs/status.md by the steward, not by this PR.
- [x] `make check` green; reviewers approve.

## Out of scope
- New corpora.

## Outcome
- Added `eval/roundtrip/{calipers,poi,corpus-bench}`. Each reads its cell suite's fetched cache and is skipped,
  with a message, while that cache is absent. The calipers fetch now also takes `roundtrip/**/init.xlsx`, the
  authored inputs; the cell suite still scores only the goldens.
- The corpus suites use `baseline = "cases"`: one `* <n>` line per failing case, where n is its failing items, and
  `roundtrip::diff` lets n only fall. Cells mode would have committed about 550,000 lines.
- The CLI line and `roundtrip.md` now give each suite's kept rate (`KeptRate`, SPEC-0033 D8). It is the figure
  M15's goal is stated in.
- First committed numbers, with main before T-4559 (every save regenerated):

  | suite | cases | items | kept |
  |---|---|---|---|
  | roundtrip-calipers | 541 | 50,833,581 | 99.84% |
  | roundtrip-poi | 350 | 6,387,847 | 99.34% |
  | roundtrip-corpus-bench | 912 | 12,637,690 | 99.00% |
  | roundtrip-fixtures | 25 | 18,796 | 90.27% |

- Not done: scoring `roundtrip/` against Excel's golden (D8's `golden` key) still needs the `golden` manifest key.
  The `init.xlsx` inputs are scored by R0 like any other case.
- The three corpora add about 3 minutes to `make eval` when fetched.
