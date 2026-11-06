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

PR #3480 (T-4456) reviews, MINORs (wording and metadata only, no engine change):
- probes/t4456-beta-t-zero-cut and -reach: their `description` and `tags` still say T-4254 (they were salvaged from
  T-4254's branch). Say T-4456 (a description or tag change refills nothing; refresh the `.math.json` sidecars with
  `rustsheet-accuracy math` if their hash moves).
- T-4463's task file mentions decision 0188 R3, but its residue is far below R3's 1E-4 relative error: drop the mention.
- statistics.md "Where the exact 0 starts (T-4045)", "More shapes":
  - cite the source of "the math there is 1.05E-264": the case's math sidecar
    (t4456-beta-t-zero-cut.math.json C10, 1.0547767311669979E-264);
  - "Excel is the math to 14 digits from 7.8E-209 down to 8.5E-291" overstates it: reach A2, A11 and A20 are 1.1E-13,
    1.2E-13 and 2.7E-13 relative off, about 12 to 13 digits (T-4463's residue). Say so;
  - the earlier cut's upper bound tightens to 5.45E-291: reach C1, `BETA.DIST(0.793650881331298,3000,5.5,TRUE)`, is a
    number there;
  - "(reach F1, where t² overflows)" is unprobed: mark it a hypothesis.

## Acceptance criteria
- [ ] The three statements above are fixed in the entry, each citing the cells.
- [ ] PR #3480's MINORs above are fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
