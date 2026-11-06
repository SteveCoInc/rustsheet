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
  with a message, while that cache is absent. A gated suite with no root fails `--check` and `--update-baseline`.
  The calipers fetch now also takes `roundtrip/**/init.xlsx`, the authored inputs; the cell suite still scores only
  the goldens.
- Items now carry a `weight`, the source items they cover that were not kept:
  - a lost element weighs its subtree;
  - a lost XML part weighs its elements, which `checked` now counts too;
  - `<truncated>` weighs the differences it hides.

  Round 1 counted one per item. The excel-fidelity review showed that hid more than 600,000 differences on Calipers.
- The corpus suites use `baseline = "cases"`: one `* <n> <digest>` line per failing case. Cells mode would have
  committed about 550,000 lines. `roundtrip::diff`:
  - lets n only fall;
  - regresses on a held n with another item set, on a fall in a case's checked count, and on a case that newly
    does not run;
  - refuses to drop cases from a `cases` suite.
- The CLI line and `roundtrip.md` give each suite's kept rate and its unscored cases (`KeptRate`, SPEC-0033 D8). M15's
  goal is stated in this rate.
- First committed numbers, on main with T-4559's verbatim save. Lost subtrees and unreadable parts now weigh
  their attributes too (round 2):

  | suite | cases (not scored) | failing cases | items | kept |
  |---|---|---|---|---|
  | roundtrip-calipers | 541 (0) | 106 | 57,554,990 | 92.04% |
  | roundtrip-poi | 350 (14) | 60 | 6,823,095 | 96.78% |
  | roundtrip-corpus-bench | 912 (0) | 162 | 14,671,895 | 94.71% |
  | roundtrip-fixtures | 25 (0) | 25 | 20,867 | 78.37% |

  Every workbook that saves verbatim keeps 100%. The losses come from the failing cases, whose saves regenerate
  (T-4560 patches cells instead). In the fixtures, the theme's `themeElements` (lost when regenerated) is the
  largest single loss.
- Not done: scoring `roundtrip/` against Excel's golden (D8's `golden` key) needs that manifest key. The
  `init.xlsx` inputs are scored by R0 like any other case.
- When fetched, the three corpora add about 3 minutes to `make eval`. They are `gate = false`.
  `rustsheet-eval roundtrip --check --suite <name>` gates one on demand.
