---
id: T-4951
title: "Probe RANK.AVG tie averaging over a lifted OFFSET or INDIRECT ref (T-4801 follow-up)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's RANK.AVG values with tied values in a lifted OFFSET or INDIRECT element's range are known, and so is how a lifted `number` or `order` pairs with a lifted `ref` (zipped or broadcast); ours matches or a fix task exists.

## Context
T-4801 probed the grids with no ties, so RANK.AVG equalled RANK.EQ everywhere. Probe a tie inside one element's range (excel-probe skill), same-sheet and off-sheet; record in docs/excel-semantics/statistics.md.

T-4801's lifted `number` ({5;9}) and `order` ({0;1}) shapes could not tell zipping from broadcasting (reviewer-excel-fidelity on #3979). Add discriminating shapes over Data A1:A3 = 5,8,2: `RANK({5;8},OFFSET(A1,{0;1},0,2))` (zip {2;1}, broadcast {2;#N/A}) and `RANK(8,OFFSET(A1,{0;1},0,2),{0;1})` (zip {1;2}, broadcast-0 {1;1}), for RANK, RANK.EQ and RANK.AVG, and replace the UNVERIFIED sentence in docs/excel-semantics/lookup.md's T-4801 entry with the result.

## Acceptance criteria
- [ ] The zip-or-broadcast shapes above are filled and lookup.md's T-4801 entry records the result.
- [ ] A probe case filled on excel-graph; ours matches every cell or each mismatch has a fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' lifted-argument pairing.
