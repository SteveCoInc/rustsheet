---
id: T-4075
title: "T-3886 research note: correct the variant counts and A/B wording, commit the shape-of-1 instrument and cell_diff output, and analyse ftest-decomposition B12"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3886]
---

## Goal
T-3886's research note (docs/research/t3886) and its statistics.md entries say exactly what was measured, and every
count can be reproduced from committed files.

## Context
These are the MINOR review findings on PR #3123 (T-3886), deferred because every reviewer had approved:
- `docs/research/t3886/score_half_x87.py`: the docstring says "96 ... variants" and "11 double-only closed forms".
  The script actually scores 192 precision variants (2^4 x 3 x 4) and 10 double forms, as its .out file shows.
- RESULTS.md section 2, the A/B bullets: the per-function "better/worse" counts are cells whose relative error to Excel
  went down or up (`docs/research/t3007/cell_diff.py`, 134 changed cells, 81 better and 53 worse). The headline
  "78 become bit-exact, 47 lose" counts bit-exactness. Say which is which so the numbers reconcile.
- RESULTS.md section 2: the per-cell "matched by" table came from a test-only instrument in `shape_one_tails` that was
  never committed, and the A/B's cell_diff output was not committed either. Commit the instrument as a `.patch`
  (a test that dumps x0, b0, l, p and w for the five cells and tries each expm1 variant at l and p shifted by up to
  2 ulp), and commit the cell_diff summary as a `.out`.
- probes/ftest-decomposition B12 is one of the 6 cells but was not analysed (its x is formed by the caller). The
  statistics.md entry "Incomplete beta at a shape of 1 outside the closed forms" and T-3886's Outcome say "no form
  of w fits them". Either analyse B12 the same way or narrow the wording to the 5 analysed cells.
- The T-3886 task file's acceptance boxes are unticked. Tick them, noting that the two split items went to T-4070.

## Acceptance criteria
- [ ] The docstring, RESULTS.md and statistics.md counts agree with the scripts' output.
- [ ] The instrument patch and the cell_diff output are committed under docs/research/t3886.
- [ ] B12 is analysed, or the claim is narrowed to the 5 cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting any form of w (T-3764 for the t3568 cells).
