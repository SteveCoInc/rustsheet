---
id: T-4457
title: "Tighten T-4223's statistics.md entry: numeric text probed in the second test array only, criteria that match nothing, inferred Z.TEST text"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4223]
---

## Goal
docs/excel-semantics/statistics.md "M9 statistical argument kinds of an array, aggregate or criteria parameter
(T-4223)" claims only what probes/t4223-m9-statistical-array-kinds-a and -b show.

## Context
PR #3475 (T-4223) excel-fidelity review, three MINORs (wording only, no engine change):
- "Array parameters given a scalar": CHISQ.TEST, CHITEST, T.TEST and TTEST are `#N/A` for `""` and `#N/A` in either
  array, but numeric text was probed as the second array only (a A14, b A34). Say so.
- "Criteria": the data holds only numbers, so the probe shows only criteria that match nothing (0, -1, 0.5, TRUE,
  `#N/A`: AVERAGEIF(S) `#DIV/0!`, MAXIFS and MINIFS 0). Drop "a number, a boolean or an error criterion matches cells
  holding that value" and record that a TRUE criterion does not match the number 1 in B1 (a A10, b A1, A8).
- "Ours": `z_test.rs` reads direct data text through `Model::cast_number`, so date, currency, percent and fraction text
  are read as numbers by inference. Only `"3"`, `""` and `"a"` are probed. Mark that UNVERIFIED.

## Acceptance criteria
- [ ] The three statements above are fixed in the entry, each citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
