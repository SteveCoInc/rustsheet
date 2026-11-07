---
id: T-3612
title: "Gate A defs: one smaller-tail quantile helper and a shared DF_CUT"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The Gate A definitions (`tools/src/rustsheet_tools/accuracy/defs/`) solve a quantile from its smaller tail in one
shared helper, take the CHISQ.INV df limit from one shared place, and compute the series tolerance once.

## Context
- Review minors on T-3529 (PR #2668), deferred so the reviewed head did not move:
  - `fisher.quantile` (fisher.py), `gamma.quantile` (gamma.py) and `beta.quantile` (beta.py) repeat the same
    skeleton: pick the smaller tail, then solve ln tail(e^s) = ln q with `solve_log_bracket`. Only the tail functions
    and the bracket differ.
  - gamma.py imports `DF_CUT` from student_t.py, so the chi-squared family depends on the Student-t family for a
    statistics.md constant.
  - gamma.py's `_series` and `_continued_fraction` each compute `eps = 2 ** (-prec + 4)` with the same comment.
- Pure refactor: every generated table and sidecar must stay byte-identical.

## Acceptance criteria
- [x] One helper in `accuracy/model.py` (next to `solve_log_bracket`) takes the lower and upper tail functions, p and
      the bracket; fisher, gamma and beta call it.
- [x] `DF_CUT` lives in a shared module, and student_t.py and gamma.py import it from there.
- [x] The series tolerance is computed by one helper in gamma.py.
- [x] `rustsheet-accuracy generate` for an existing table reproduces the committed `s1.math.json` byte for byte (for
      F.INV, GAMMA.INV and BETA.DIST at least), and `tools/tests` pass unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- New definitions or changing any value.

## Outcome
`solve_smaller_tail` (accuracy/model.py, next to `solve_log_bracket`) now serves fisher, gamma and beta (beta's upper
branch is the lower tail of 1 - x); `DF_CUT` lives in defs/_common.py (student_t, fisher and gamma import it
there); gamma.py computes the series tolerance in `_eps`. `rustsheet-accuracy verify` gives 0 differences for F.INV,
GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, BETA.INV and BETA.DIST; `tools/tests` pass unchanged (399 accuracy tests). No eval
cell changes.
