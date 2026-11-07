---
id: T-4924
title: "No row-label column for a non-compact pivot with no row field and Values on columns"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An outline or tabular pivot with no row field and the Values field on columns has no row-label column, as Excel lays it out: its first column is the first data column.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

gen-pivot/640780d7b6f4ad53 P02 (outline, column field Rep, three data fields, Values last on columns) is L5:DC8 in Excel and L5:DD8 in the engine, which keeps an empty first column; gen-pivot/f9e7edbb5b74011a P07 (tabular, L5:CT9 against L5:CU9) and gen-pivot/462f10551d480f9d P08 (tabular, Values first on columns, L5:W9 against L5:X9) are the same. Every cell of those pivots is one column off. With one data field (no Values field) the engine already matches (gen-pivot/01865529b5a09c0b P03). Related probe: T-4650.

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/lines.rs (`geometry`), crates/rustsheet-engine/src/pivot/layout/render.rs. The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 465 values and texts on 3 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/462f10551d480f9d P08!L3:W9
- gen-pivot/640780d7b6f4ad53 P02!L5:DC8
- gen-pivot/f9e7edbb5b74011a P07!L5:CT9

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
