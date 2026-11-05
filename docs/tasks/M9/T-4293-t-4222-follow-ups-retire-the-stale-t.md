---
id: T-4293
title: "T-4222 follow-ups: retire the stale T-3185 two-cell-range bullet and probe a blank const for LOGEST and GROWTH and a computed blank"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4222]
---

## Goal
statistics.md says one thing about a range or blank cell as a regression const or stats, and every member of the
family has evidence for the blank-cell rule.

## Context
- MINOR findings of T-4222's Excel-fidelity review (PR #3285), deferred as follow-ups.
- The T-3185 entry "M9 statistical argument kinds of a scalar parameter" in docs/excel-semantics/statistics.md still
  says, in its two-cell-range bullet, that the shape is unprobed (T-4222), that ours spills 1-by-2, and that ours is
  `#VALUE!` for GROWTH's and TREND's `const`. T-4222's entry "A range as const or stats reads its top-left cell"
  supersedes it.
- That entry's "a blank cell is FALSE" is evidenced for TREND's and LINEST's const (probes/t4222-regression-flag-ranges
  A151, A154, A127, A157) and LOGEST's stats (T-3185); a blank const for LOGEST and GROWTH, and a computed blank (e.g.
  `IF(TRUE,Data!Z151)`, a LAMBDA returning a blank cell) are unprobed. They share `flag_value` and TREND's kernel.

## Acceptance criteria
- [ ] The T-3185 bullet points at the T-4222 entry and says ours matches.
- [ ] A probe (excel-graph) covers a blank const for LOGEST and GROWTH and a computed blank for one member of each
      code path; the T-4222 entry cites it, and any mismatch is fixed or owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The base calls' last-digit residue.
